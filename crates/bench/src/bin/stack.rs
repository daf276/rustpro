// The full simulation stack on Elfnote mirror matches under uniformly random legal actions, setup excluded, on one
// thread and on `--threads`: play, the calls a search or RL loop makes at a prompt, and the batch runner, beside the
// phase 0 target and earlier measurements from PLAN.md. Build with `--features mimalloc` to compare allocators.
use std::alloc::{GlobalAlloc, Layout};
use std::cell::Cell;
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

use bench::{BenchResult, Decks, play, run_batch, setup};
use carddata::{CardDatabase, Deck};
use engine::{Duel, Prompt, Xoshiro256StarStar};
use rayon::prelude::*;
use sim::{ACTION_FEATURES, ACTIONS, AutoAdvance, CARD_FEATURES, CHAIN_FEATURES, GLOBAL_FEATURES, MAX_CHAIN, MAX_OBS_CARDS, Tensors};

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc as Inner;
#[cfg(not(feature = "mimalloc"))]
use std::alloc::System as Inner;

struct Counting;

thread_local! {
    // Allocations and bytes this thread asked for.
    static ALLOCATED: Cell<[u64; 2]> = const { Cell::new([0; 2]) };
}

fn count(bytes: usize) {
    let _ = ALLOCATED.try_with(|allocated| {
        let [count, total] = allocated.get();
        allocated.set([count + 1, total + bytes as u64]);
    });
}

fn allocated() -> [u64; 2] {
    ALLOCATED.try_with(Cell::get).unwrap_or_default()
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { Inner.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { Inner.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count(size);
        unsafe { Inner.realloc(ptr, layout, size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { Inner.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const SAMPLE_EVERY: u64 = 16;
const LEGAL_REPEATS: u32 = 16;
const SPIN: u64 = 50_000_000;
const OPS: [&str; 6] = [
    "legal_actions_into",
    "clone",
    "observe",
    "encode",
    "observe into tensors",
    "determinize",
];

// Per call: the time (its result dropped included), allocations and bytes allocated.
#[derive(Default)]
struct Op {
    nanos: Vec<u32>,
    allocations: u64,
    bytes: u64,
    calls: u64,
}

impl Op {
    fn time<T>(&mut self, repeats: u32, mut call: impl FnMut() -> T) {
        let before = allocated();
        let started = Instant::now();
        for _ in 0..repeats {
            black_box(call());
        }
        let nanos = started.elapsed().as_nanos() / u128::from(repeats);
        let after = allocated();
        self.nanos.push(u32::try_from(nanos).unwrap_or(u32::MAX));
        self.allocations += after[0] - before[0];
        self.bytes += after[1] - before[1];
        self.calls += u64::from(repeats);
    }

    fn merge(mut self, mut other: Self) -> Self {
        self.nanos.append(&mut other.nanos);
        self.allocations += other.allocations;
        self.bytes += other.bytes;
        self.calls += other.calls;
        self
    }

    fn quantile(&self, quantile: f64) -> u32 {
        let at = ((self.nanos.len() as f64 - 1.0) * quantile) as usize;
        self.nanos.get(at).copied().unwrap_or(0)
    }
}

type Ops = [Op; OPS.len()];

// The calls at every SAMPLE_EVERY-th prompt of one duel: the legal actions, a clone, both players' observations and
// their tensors, both written straight from the duel, and a determinization for the player prompted, with the deck as
// the opponent's decklist.
fn sample(seed: u64, cards: &'static CardDatabase, decks: &Decks, deck: &Deck) -> BenchResult<Ops> {
    let (mut duel, mut rng) = setup(seed, cards, decks)?;
    let mut ops = Ops::default();
    let mut legal = Vec::new();
    let mut sampler = Xoshiro256StarStar::new([seed, 1, 2, 3]);
    let (mut global, mut seen, mut chain) = (
        vec![0; GLOBAL_FEATURES.len()],
        vec![0; MAX_OBS_CARDS * CARD_FEATURES.len()],
        vec![0; MAX_CHAIN * CHAIN_FEATURES.len()],
    );
    let mut actions = vec![0; ACTIONS * ACTION_FEATURES.len()];
    let mut tensors = Tensors::new(&mut global, &mut seen, &mut chain, &mut actions)?;
    play(&mut duel, &mut rng, |duel, _, action| {
        let [legal_op, clone, observe, encode, straight, determinize] = &mut ops;
        if action % SAMPLE_EVERY != 0 {
            return;
        }
        legal_op.time(LEGAL_REPEATS, || duel.legal_actions_into(&mut legal));
        clone.time(1, || duel.clone());
        for player in 0..2 {
            observe.time(1, || duel.observe(player));
            let observation = duel.observe(player);
            encode.time(1, || tensors.write(0, Some(&observation)));
            straight.time(1, || tensors.write_duel(0, duel, player));
        }
        let player = duel.prompt().map_or(0, Prompt::player);
        determinize.time(1, || duel.determinize(player, &mut sampler, deck));
    })?;
    Ok(ops)
}

fn sample_all(threads: usize, seeds: std::ops::Range<u64>, cards: &'static CardDatabase, decks: &Decks) -> BenchResult<Ops> {
    let deck = decks.deck.as_ref().ok_or("no deck")?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let merge = |ours: Ops, theirs: Ops| {
        let mut theirs = theirs.into_iter();
        Ok(ours.map(|op| op.merge(theirs.next().unwrap_or_default())))
    };
    let mut ops = pool.install(|| {
        seeds
            .into_par_iter()
            .map(|seed| sample(seed, cards, decks, deck))
            .try_reduce(Ops::default, merge)
    })?;
    for op in &mut ops {
        op.nanos.sort_unstable();
    }
    Ok(ops)
}

// Actions, actions at prompts with more than one legal action, allocations, and the seconds taken.
fn apply(threads: usize, seeds: std::ops::Range<u64>, cards: &'static CardDatabase, decks: &Decks) -> BenchResult<([u64; 3], f64)> {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let duels = seeds
        .into_par_iter()
        .map(|seed| setup(seed, cards, decks))
        .collect::<Result<Vec<_>, _>>()?;
    let started = Instant::now();
    let counts = pool.install(|| {
        duels
            .into_par_iter()
            .map(|(mut duel, mut rng)| -> BenchResult<[u64; 3]> {
                let ([before, _], mut decisions) = (allocated(), 0);
                let actions = play(&mut duel, &mut rng, |_, legal, _| decisions += u64::from(legal.len() > 1))?;
                Ok([actions, decisions, allocated()[0] - before])
            })
            .try_reduce(
                || [0; 3],
                |ours, theirs| {
                    let mut theirs = theirs.into_iter();
                    Ok(ours.map(|count| count + theirs.next().unwrap_or(0)))
                },
            )
    })?;
    Ok((counts, started.elapsed().as_secs_f64()))
}

// Rounds per second of a loop that touches no memory, with four independent chains so that a busy sibling hyperthread
// slows it: how far the machine scales at the moment of the run.
fn spin(threads: usize) -> BenchResult<f64> {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let rounds = 4 * threads as u64;
    let started = Instant::now();
    let sum: u64 = pool.install(|| {
        (0..rounds)
            .into_par_iter()
            .map(|round| {
                let mut chains = [round | 1, round | 2, round | 4, round | 8];
                for _ in 0..SPIN {
                    for x in &mut chains {
                        *x ^= *x << 13;
                        *x ^= *x >> 7;
                        *x ^= *x << 17;
                    }
                }
                chains.iter().fold(0, |sum, x| sum ^ x)
            })
            .sum()
    });
    black_box(sum);
    Ok(rounds as f64 / started.elapsed().as_secs_f64())
}

fn kilo(value: f64) -> String {
    if value >= 1e6 {
        format!("{:.2}M", value / 1e6)
    } else {
        format!("{:.1}k", value / 1e3)
    }
}

fn row(out: &mut impl Write, name: &str, one: String, many: String, scaling: String, target: &str, earlier: &str) -> std::io::Result<()> {
    writeln!(out, "{name:<34}{one:>22}{many:>22}{scaling:>8}  {target:<24}{earlier}")
}

fn main() -> BenchResult<()> {
    let (mut duels, mut threads, mut seed, mut batch, mut sampled) = (2000, 7, 3, 64, 500);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value: u64 = args.next().ok_or("missing value")?.parse()?;
        match flag.as_str() {
            "--duels" => duels = value,
            "--threads" => threads = usize::try_from(value)?,
            "--seed" => seed = value,
            "--batch" => batch = usize::try_from(value)?,
            "--sampled" => sampled = value,
            _ => return Err(format!("unknown argument {flag}, expected --duels, --threads, --seed, --batch or --sampled").into()),
        }
    }
    let (cards, decks) = Decks::load(false, true)?;
    let seeds = seed..seed + duels;
    let out = &mut std::io::stdout().lock();
    let allocator = if cfg!(feature = "mimalloc") { "mimalloc" } else { "system" };
    writeln!(
        out,
        "Elfnote mirror, seeds {seeds:?}, {allocator} allocator, size_of::<Duel>() = {} B",
        size_of::<Duel>()
    )?;
    writeln!(
        out,
        "{:<34}{:>22}{:>22}{:>8}  {:<24}earlier",
        "",
        "1 thread",
        format!("{threads} threads"),
        "x",
        "target"
    )?;

    let [(one, one_seconds), (many, many_seconds)] = [
        apply(1, seeds.clone(), cards, &decks)?,
        apply(threads, seeds.clone(), cards, &decks)?,
    ];
    let rate = |count: u64, seconds: f64| kilo(count as f64 / seconds);
    let scaling = format!("{:.2}", one_seconds / many_seconds);
    let earlier = "2e 256-258k, 5b 240k; 5c 234-240k / 0.96-1.16M";
    row(
        out,
        "apply, actions/s",
        rate(one[0], one_seconds),
        rate(many[0], many_seconds),
        scaling.clone(),
        "1M/thread, ~linear",
        earlier,
    )?;
    let decisions = "apply, decisions/s (>1 legal)";
    row(
        out,
        decisions,
        rate(one[1], one_seconds),
        rate(many[1], many_seconds),
        scaling,
        "",
        "",
    )?;
    let per_action = |counts: [u64; 3]| format!("{:.3}", counts[2] as f64 / counts[0] as f64);
    row(
        out,
        "apply, allocations/action",
        per_action(one),
        per_action(many),
        String::new(),
        "",
        "2e 0.54",
    )?;

    let sampled = seed..seed + duels.min(sampled);
    let [one, many] = [
        sample_all(1, sampled.clone(), cards, &decks)?,
        sample_all(threads, sampled, cards, &decks)?,
    ];
    let latency = |op: &Op| format!("{} / {} ns", op.quantile(0.5), op.quantile(0.99));
    let earlier = [
        "",
        "5b mean 4.7-4.95 us in play",
        "5b median 4.7 us",
        "5d observe+encode ~5 us",
        "5e observe 2.8 us, est. ~1.5 us",
        "5b median 22.5 us",
    ];
    let targets = ["", "<= ~1 us", "", "", "", ""];
    for (((name, (one, many)), target), earlier) in OPS.iter().zip(one.iter().zip(&many)).zip(targets).zip(earlier) {
        row(
            out,
            &format!("{name}, median / p99"),
            latency(one),
            latency(many),
            String::new(),
            target,
            earlier,
        )?;
        let calls = one.calls.max(1) as f64;
        let per_call = format!(
            "{:.2} / {:.1} KB",
            one.allocations as f64 / calls,
            one.bytes as f64 / calls / 1024.0
        );
        row(
            out,
            &format!("{name}, allocations / KB"),
            per_call,
            String::new(),
            String::new(),
            "",
            "",
        )?;
    }
    writeln!(
        out,
        "({} calls of each on 1 thread, legal_actions_into timed over {LEGAL_REPEATS} calls)",
        one[1].calls
    )?;

    let modes = [
        (AutoAdvance::Off, "5c 158k / 734k; 5d 133k / 513-518k"),
        (AutoAdvance::Own, "5c 158k / 781k, 135k / 667k decisions"),
        (AutoAdvance::Both, "5c 190k / 839k, 88k / 389k decisions"),
    ];
    for (mode, earlier) in modes.into_iter().filter(|_| batch > 0) {
        let [
            (one_actions, one_decisions, one_seconds),
            (many_actions, many_decisions, many_seconds),
        ] = [
            run_batch(1, seeds.clone(), cards, &decks, batch, mode)?,
            run_batch(threads, seeds.clone(), cards, &decks, batch, mode)?,
        ];
        let name = format!("batch {batch} {mode:?}, actions/s");
        let scaling = format!("{:.2}", one_seconds / many_seconds);
        let [one, many] = [rate(one_actions, one_seconds), rate(many_actions, many_seconds)];
        row(out, &name, one, many, scaling, "", earlier)?;
        let [one, many] = [rate(one_decisions, one_seconds), rate(many_decisions, many_seconds)];
        row(
            out,
            &format!("batch {batch} {mode:?}, decisions/s"),
            one,
            many,
            String::new(),
            "",
            "",
        )?;
    }

    let [one, many] = [spin(1)?, spin(threads)?];
    let rounds = |rate: f64| format!("{rate:.1}");
    row(
        out,
        "spin loop, rounds/s",
        rounds(one),
        rounds(many),
        format!("{:.2}", many / one),
        "~linear",
        "",
    )?;
    Ok(())
}
