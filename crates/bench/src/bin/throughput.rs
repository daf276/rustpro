use std::hint::black_box;
use std::io::Write;
use std::time::{Duration, Instant};

use bench::{BenchResult, Decks, MAX_ACTIONS, play, run_batch, setup};
use carddata::CardDatabase;
use engine::Duel;
use rand::rngs::StdRng;
use rayon::prelude::*;
use sim::AutoAdvance;

const CLONE_DUELS: u64 = 500;
const CLONE_EVERY: u64 = 16;
const CLONES: u32 = 32;
const WIN_BY_LP: u8 = 1;

struct Run {
    threads: usize,
    seconds: f64,
    actions: u64,
    by_lp: usize,
    capped: usize,
    unported: usize,
}

fn run(threads: usize, seeds: std::ops::Range<u64>, cards: &'static CardDatabase, decks: &Decks) -> BenchResult<Run> {
    let threads = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let duels: Vec<(Duel, StdRng)> = seeds
        .into_par_iter()
        .map(|seed| setup(seed, cards, decks))
        .collect::<Result<_, _>>()?;
    let started = Instant::now();
    let played = threads.install(|| {
        duels
            .into_par_iter()
            .map(|(mut duel, mut rng)| {
                Ok((
                    play(&mut duel, &mut rng, |_, _, _| {})?,
                    duel.result(),
                    duel.unported_site().is_some(),
                ))
            })
            .collect::<BenchResult<Vec<_>>>()
    })?;
    Ok(Run {
        threads: threads.current_num_threads(),
        seconds: started.elapsed().as_secs_f64(),
        actions: played.iter().map(|(actions, ..)| actions).sum(),
        by_lp: played
            .iter()
            .filter(|(_, result, _)| result.is_some_and(|(_, reason)| reason == WIN_BY_LP))
            .count(),
        capped: played.iter().filter(|&&(actions, ..)| actions >= MAX_ACTIONS).count(),
        unported: played.iter().filter(|&&(.., unported)| unported).count(),
    })
}

// Mean time of a clone (and its drop) at every CLONE_EVERY-th prompt of the first CLONE_DUELS duels.
fn clone_time(seeds: std::ops::Range<u64>, cards: &'static CardDatabase, decks: &Decks) -> BenchResult<(Duration, u32)> {
    let (mut total, mut clones) = (Duration::ZERO, 0);
    for seed in seeds {
        let (mut duel, mut rng) = setup(seed, cards, decks)?;
        play(&mut duel, &mut rng, |duel, _, action| {
            if action % CLONE_EVERY == 0 {
                let started = Instant::now();
                for _ in 0..CLONES {
                    black_box(black_box(duel).clone());
                }
                total += started.elapsed();
                clones += CLONES;
            }
        })?;
    }
    Ok((total / clones.max(1), clones))
}

// Engine throughput under uniformly random legal actions in random Normal Monster duels, setup excluded: one thread,
// then `--threads` (0 = all cores). `--effects 1` mixes the cards of phase 2a into the decks; `--elfnote 1` plays
// Elfnote mirror matches (main and Extra Deck) instead; `--batch N` also plays them through the batch runner with N slots.
// "Unported" counts the duels whose result cannot be trusted, since an effect reached an unported call site.
fn main() -> BenchResult<()> {
    let (mut duels, mut threads, mut seed, mut effects, mut elfnote, mut batch) = (5000, 0, 0, 0, 0, 0);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value: u64 = args.next().ok_or("missing value")?.parse()?;
        match flag.as_str() {
            "--duels" => duels = value,
            "--threads" => threads = usize::try_from(value)?,
            "--seed" => seed = value,
            "--effects" => effects = value,
            "--elfnote" => elfnote = value,
            "--batch" => batch = usize::try_from(value)?,
            _ => {
                let expected = "--duels, --threads (0 = all cores), --seed, --effects, --elfnote or --batch";
                return Err(format!("unknown argument {flag}, expected {expected}").into());
            }
        }
    }
    let (cards, decks) = Decks::load(effects != 0, elfnote != 0)?;
    let seeds = seed..seed + duels;

    let out = &mut std::io::stdout().lock();
    writeln!(out, "size_of::<Duel>() = {} B", size_of::<Duel>())?;
    let (clone, clones) = clone_time(seed..seed + duels.min(CLONE_DUELS), cards, &decks)?;
    writeln!(out, "clone: {} ns mean over {clones} clones", clone.as_nanos())?;
    let single = run(1, seeds.clone(), cards, &decks)?;
    let parallel = run(threads, seeds.clone(), cards, &decks)?;
    writeln!(
        out,
        "{duels} duels (seeds {seeds:?}), {} actions ({:.0} per duel), {} won by LP, {} capped at {MAX_ACTIONS} actions, {} unported",
        single.actions,
        single.actions as f64 / duels as f64,
        single.by_lp,
        single.capped,
        single.unported
    )?;
    for run in [&single, &parallel] {
        writeln!(
            out,
            "{:>2} threads: {:>10.0} actions/s, {:>8.0} duels/s, {:.2}x one thread",
            run.threads,
            run.actions as f64 / run.seconds,
            duels as f64 / run.seconds,
            single.seconds / run.seconds
        )?;
    }
    let modes = if batch > 0 {
        [AutoAdvance::Off, AutoAdvance::Own, AutoAdvance::Both].as_slice()
    } else {
        &[]
    };
    for &auto_advance in modes {
        for threads in [1, parallel.threads] {
            let (actions, decisions, seconds) = run_batch(threads, seeds.clone(), cards, &decks, batch, auto_advance)?;
            writeln!(
                out,
                "batch of {batch}, auto-advance {auto_advance:?}, {threads:>2} threads: {:>10.0} actions/s, {:>10.0} decisions/s, {:.2}x one thread single",
                actions as f64 / seconds,
                decisions as f64 / seconds,
                actions as f64 / seconds / (single.actions as f64 / single.seconds)
            )?;
        }
    }
    Ok(())
}
