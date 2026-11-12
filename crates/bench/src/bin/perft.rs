// Game-tree size from the start of a duel counted as chess perft does, and the branching of random playouts.
use std::f64::consts::LN_10;
use std::io::Write;
use std::time::Instant;

use bench::{Allocator, BenchResult, flags, load_decks, play, setup};
use carddata::{CardDatabase, Decks};
use engine::Action;
use fiber::Fiber;
use sim::{AutoAdvance, auto_advance};

#[global_allocator]
static GLOBAL: Allocator = Allocator;

// Leaves `depth` plies below `duel`, whose prompt offers `legal`; as in chess, a duel ending earlier adds no leaves.
fn perft(duel: &Fiber, legal: &[Action], depth: u32, mode: AutoAdvance) -> BenchResult<u64> {
    if depth <= 1 {
        return Ok(legal.len() as u64);
    }
    let mut next = Vec::new();
    legal.iter().try_fold(0, |nodes, &action| -> BenchResult<u64> {
        let mut child = duel.clone();
        child.apply(action)?;
        auto_advance(&mut child, mode, 0, &mut next)?;
        Ok(nodes + perft(&child, &next, depth - 1, mode)?)
    })
}

fn perft_table(out: &mut impl Write, mut root: Fiber, seconds: f64, mode: AutoAdvance) -> BenchResult<()> {
    root.process();
    let mut legal = Vec::new();
    auto_advance(&mut root, mode, 0, &mut legal)?;
    let mut previous = 1;
    for depth in 1.. {
        let started = Instant::now();
        let nodes = perft(&root, &legal, depth, mode)?;
        let elapsed = started.elapsed().as_secs_f64();
        let growth = nodes as f64 / previous as f64;
        writeln!(out, "{depth:>6}{nodes:>16}{growth:>9.2}{elapsed:>10.2}")?;
        if elapsed > seconds || nodes == 0 {
            return Ok(());
        }
        previous = nodes;
    }
    Ok(())
}

// Per decision of `duels` uniformly random playouts, the actions a policy chooses among.
fn branching(out: &mut impl Write, seeds: std::ops::Range<u64>, cards: &'static CardDatabase, decks: &Decks) -> BenchResult<()> {
    let (mut options, mut policy, started) = (Vec::new(), Vec::new(), Instant::now());
    let duels = seeds.end.saturating_sub(seeds.start).max(1) as f64;
    for seed in seeds {
        let (duel, mut rng) = setup(seed, cards, decks)?;
        play(&mut Fiber::new(duel), &mut rng, |duel, _, _| {
            duel.policy_actions_into(&mut policy);
            options.push(policy.len());
        })?;
    }
    options.sort_unstable();
    let logs: Vec<f64> = options
        .iter()
        .filter(|&&count| count > 1)
        .map(|&count| (count as f64).ln())
        .collect();
    let decisions = options.len().max(1) as f64;
    writeln!(
        out,
        "{:>10.1}{:>12.1}{:>8.2}{:>8}{:>8.1}%{:>12.2}{:>12.0}{:>10.1}",
        decisions / duels,
        logs.len() as f64 / duels,
        options.iter().sum::<usize>() as f64 / decisions,
        options.get(options.len() / 2).copied().unwrap_or(0),
        100.0 * (1.0 - logs.len() as f64 / decisions),
        (logs.iter().sum::<f64>() / logs.len().max(1) as f64).exp(),
        logs.iter().sum::<f64>() / LN_10 / duels,
        started.elapsed().as_secs_f64(),
    )?;
    Ok(())
}

// `--seconds` stops a perft table after the first depth that took longer, `--duels` sets the playouts per deck.
fn main() -> BenchResult<()> {
    let [seed, seconds, duels] = flags(["--seed", "--seconds", "--duels"], [3, 60, 200])?;
    let out = &mut std::io::stdout().lock();
    writeln!(
        out,
        "Seed {seed}. The duel's RNG is part of the state, so shuffles, draws and coin flips are fixed and the tree is deterministic.\n\
         A ply is one decision of the prompted player, each legal action a branch; nodes at a depth are the states reached in exactly that many plies."
    )?;
    let decks = [("Normal Monster decks", false), ("Elfnote mirror", true)];
    let modes = [
        (AutoAdvance::Off, "every prompt a ply"),
        (AutoAdvance::Both, "single-option prompts auto-advanced, not plies"),
    ];
    for (name, elfnote) in decks {
        let (cards, decks) = load_decks(false, elfnote)?;
        for (mode, plies) in modes {
            writeln!(out, "\n{name}, {plies}\n{:>6}{:>16}{:>9}{:>10}", "depth", "nodes", "x", "seconds")?;
            perft_table(out, Fiber::new(setup(seed, cards, &decks)?.0), seconds as f64, mode)?;
        }
        writeln!(
            out,
            "\n{name}, {duels} random playouts from seed {seed}; decisions per duel, options per decision as a policy sees them\n{:>10}{:>12}{:>8}{:>8}{:>9}{:>12}{:>12}{:>10}",
            "decisions", "non-forced", "mean", "median", "forced", "effective b", "log10 tree", "seconds"
        )?;
        branching(out, seed..seed + duels, cards, &decks)?;
    }
    Ok(())
}
