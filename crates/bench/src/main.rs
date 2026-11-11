use std::collections::HashMap;
use std::io::Write;
use std::time::{Duration, Instant};

use bench::{BenchResult, flags};
use carddata::{CardDatabase, Deck, Decks, ELFNOTE_DECK, load_advanced, vendor};
use engine::Duel;
use oracle::{Outcome, Report, Scripts, culprit, play_random_duel};
use rand::rngs::StdRng;
use rayon::prelude::*;

const MAX_DECISIONS: u32 = 5_000;
const TOP_ERRORS: usize = 20;

// Oracle throughput on random Advanced decks, or with `--elfnote 1` on Elfnote mirror matches the engine then replays.
fn main() -> BenchResult<()> {
    let [duels, threads, seed, elfnote] = flags(["--duels", "--threads", "--seed", "--elfnote"], [1000, 0, 0, 0])?;
    let (cards, pool) = load_advanced(&vendor())?;
    let cards: &'static CardDatabase = Box::leak(Box::new(cards));
    let scripts = Scripts::load(&vendor().join("CardScripts"))?;
    let threads = rayon::ThreadPoolBuilder::new().num_threads(usize::try_from(threads)?).build()?;
    let decks = Decks {
        pool,
        effect_cards: Vec::new(),
        deck: (elfnote != 0).then(|| Deck::parse(ELFNOTE_DECK, cards)),
    };

    let started = Instant::now();
    let reports: Vec<Report> = threads.install(|| {
        (seed..seed + duels)
            .into_par_iter()
            .map(|seed| play_random_duel(cards, &scripts, |rng: &mut StdRng| decks.deal(rng), seed, MAX_DECISIONS, |_| {}))
            .collect::<Result<_, _>>()
    })?;
    let elapsed = started.elapsed().as_secs_f64();

    let decisions: u64 = reports.iter().map(|report| u64::from(report.decisions)).sum();
    let turns: u64 = reports.iter().map(|report| u64::from(report.turns)).sum();
    let setup: Duration = reports.iter().map(|report| report.setup).sum();
    let count = |matches: fn(&Outcome) -> bool| reports.iter().filter(|report| matches(&report.outcome)).count();
    let mut culprits: HashMap<&str, (usize, &str)> = HashMap::new();
    for error in reports.iter().flat_map(|report| &report.errors) {
        culprits.entry(culprit(error).unwrap_or("(unattributed)")).or_insert((0, error)).0 += 1;
    }
    let mut culprits: Vec<_> = culprits.into_iter().collect();
    culprits.sort_by(|left, right| right.1.0.cmp(&left.1.0).then(left.0.cmp(right.0)));

    let duel_count = reports.len() as f64;
    let out = &mut std::io::stdout().lock();
    writeln!(
        out,
        "{duel_count} duels on {} threads in {elapsed:.2} s (seeds {seed}..{})",
        threads.current_num_threads(),
        seed + duels
    )?;
    writeln!(
        out,
        "duels/s: {:.1}, decisions/s: {:.0}",
        duel_count / elapsed,
        decisions as f64 / elapsed
    )?;
    let (decisions_per_duel, turns_per_duel) = (decisions as f64 / duel_count, turns as f64 / duel_count);
    let setup_per_duel = setup.as_secs_f64() * 1000.0 / duel_count;
    writeln!(
        out,
        "mean per duel: {decisions_per_duel:.1} decisions, {turns_per_duel:.1} turns, {setup_per_duel:.2} ms setup"
    )?;
    writeln!(
        out,
        "outcomes: {} won, {} ended without winner, {} hit the {MAX_DECISIONS}-decision cap, {} MSG_RETRY, {} unanswerable",
        count(|outcome| matches!(outcome, Outcome::Win(_))),
        count(|outcome| matches!(outcome, Outcome::Ended)),
        count(|outcome| matches!(outcome, Outcome::DecisionCap)),
        count(|outcome| matches!(outcome, Outcome::Retry { .. })),
        count(|outcome| matches!(outcome, Outcome::Unanswerable(_))),
    )?;
    for report in reports
        .iter()
        .filter(|report| matches!(report.outcome, Outcome::Retry { .. } | Outcome::Unanswerable(_)))
        .take(5)
    {
        writeln!(out, "  {:?}", report.outcome)?;
    }
    let errors: usize = culprits.iter().map(|(_, (count, _))| count).sum();
    let failing = reports.iter().filter(|report| !report.errors.is_empty()).count();
    writeln!(out, "lua errors: {errors} in {failing} duels from {} scripts", culprits.len())?;
    for (culprit, (count, sample)) in culprits.iter().take(TOP_ERRORS) {
        writeln!(out, "  {count:>6}  {culprit}: {}", sample.lines().next().unwrap_or_default())?;
    }
    if decks.deck.is_some() {
        // Wall time without setup, assuming setup and play share the threads in proportion to their total times.
        let play: Duration = reports.iter().map(|report| report.play).sum();
        let oracle = decisions as f64 / (elapsed * play.as_secs_f64() / (play + setup).as_secs_f64());
        let replays: Vec<(Duel, &Report)> = (seed..)
            .zip(&reports)
            .map(|(seed, report)| Ok((bench::setup(seed, cards, &decks)?.0, report)))
            .collect::<Result<_, engine::DuelError>>()?;
        let started = Instant::now();
        let replayed: Vec<(u64, bool)> = threads.install(|| replays.into_par_iter().map(replay).collect::<Result<_, _>>())?;
        let engine = decisions as f64 / started.elapsed().as_secs_f64();
        let actions: u64 = replayed.iter().map(|(actions, _)| actions).sum();
        let diverged = replayed.iter().filter(|(_, same)| !same).count();
        writeln!(
            out,
            "setup excluded: oracle {oracle:.0} decisions/s, engine {engine:.0} decisions/s ({:.0}x)",
            engine / oracle
        )?;
        writeln!(
            out,
            "engine replay: {actions} actions ({:.2} per decision), {diverged} duels with a different result",
            actions as f64 / decisions as f64
        )?;
    }
    Ok(())
}

// Applies the oracle's responses; returns the actions they decode to and whether the duel ended as the oracle's did.
fn replay((mut duel, report): (Duel, &Report)) -> Result<(u64, bool), engine::ActionError> {
    duel.process();
    let mut actions = 0;
    for response in &report.responses {
        for action in duel.decode_response(response)? {
            duel.apply(action)?;
            actions += 1;
        }
    }
    let winner = duel.result().map(|(winner, _)| winner);
    Ok((actions, matches!(report.outcome, Outcome::Win(player) if Some(player) == winner)))
}
