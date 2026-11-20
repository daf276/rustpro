use std::io::Write;
use std::time::Instant;

use bench::{Allocator, BenchResult, MAX_ACTIONS, flags, load_decks, setup};
use carddata::{CardDatabase, Decks};
use engine::Status;
use fiber::Fiber;
use rand::seq::IndexedRandom;
use sim::Tracker;

#[global_allocator]
static GLOBAL: Allocator = Allocator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Plain,
    // The message trace taken after every action.
    Traced,
    // The trace and both players' observations, what a tracker's update reads besides the messages.
    Observed,
    // The trace and a tracker per player updated after every action.
    Tracked,
}

// The actions played, seconds and trace bytes of random duels, setup excluded.
fn run(mode: Mode, seeds: std::ops::Range<u64>, cards: &'static CardDatabase, decks: &Decks) -> BenchResult<(u64, f64, usize)> {
    let duels = seeds.map(|seed| setup(seed, cards, decks)).collect::<Result<Vec<_>, _>>()?;
    let (mut actions, mut bytes, mut legal) = (0, 0, Vec::new());
    let started = Instant::now();
    for (mut duel, mut rng) in duels {
        duel.set_trace(mode != Mode::Plain);
        let mut trackers = (mode == Mode::Tracked).then(|| [0, 1].map(|player| Tracker::new(&duel, player)));
        let mut duel = Fiber::new(duel);
        let mut status = duel.process();
        let mut played = 0;
        loop {
            let messages = duel.take_trace();
            bytes += messages.len();
            if mode == Mode::Observed {
                std::hint::black_box([0, 1].map(|player| duel.observe(player)));
            }
            for tracker in trackers.iter_mut().flatten() {
                tracker.update(&messages, &duel);
            }
            if status != Status::Awaiting || played >= MAX_ACTIONS {
                break;
            }
            duel.legal_actions_into(&mut legal);
            status = duel.apply(*legal.choose(&mut rng).ok_or("no legal action")?)?;
            played += 1;
        }
        actions += played;
    }
    Ok((actions, started.elapsed().as_secs_f64(), bytes))
}

// What the message trace and the observation trackers cost per action on one thread, Elfnote duels at random.
fn main() -> BenchResult<()> {
    let [duels, seed] = flags(["--duels", "--seed"], [200, 0])?;
    let (cards, decks) = load_decks(false, true)?;
    let out = &mut std::io::stdout().lock();
    let mut plain = None;
    for mode in [Mode::Plain, Mode::Traced, Mode::Observed, Mode::Tracked] {
        let (actions, seconds, bytes) = run(mode, seed..seed + duels, cards, &decks)?;
        let micros = seconds * 1e6 / actions as f64;
        let base = *plain.get_or_insert(micros);
        writeln!(
            out,
            "{mode:?}: {actions} actions, {micros:.2} us per action (+{:.2} us, {:.0}%), {:.0} trace bytes per action",
            micros - base,
            100.0 * (micros - base) / base,
            bytes as f64 / actions as f64
        )?;
    }
    Ok(())
}
