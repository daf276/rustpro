use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use carddata::{CardDatabase, Deck, Pool, banlist_limits};
use engine::{DUEL_MODE_MR5, Duel, DuelOptions, PlayerOptions};
use oracle::{Outcome, Report, Scripts, culprit, play_random_duel};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

const MAX_DECISIONS: u32 = 5_000;
const TOP_ERRORS: usize = 20;

// Oracle throughput under random responses: random legal Advanced decks, or with `--elfnote 1` Elfnote mirror matches
// (main and Extra Deck, shuffled per duel), which the engine then replays response for response for comparison.
fn main() -> Result<(), Box<dyn Error>> {
    let (mut duels, mut threads, mut seed, mut elfnote) = (1000, 0, 0, 0);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value: u64 = args.next().ok_or("missing value")?.parse()?;
        match flag.as_str() {
            "--duels" => duels = value,
            "--threads" => threads = usize::try_from(value)?,
            "--seed" => seed = value,
            "--elfnote" => elfnote = value,
            _ => {
                let expected = "--duels, --threads (0 = all cores), --seed or --elfnote";
                return Err(format!("unknown argument {flag}, expected {expected}").into());
            }
        }
    }
    let vendor = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let cards: &'static CardDatabase = Box::leak(Box::new(CardDatabase::load(&vendor.join("BabelCDB"))?));
    let scripts = Scripts::load(&vendor.join("CardScripts"))?;
    let pool = Pool::advanced(
        cards,
        banlist_limits(&std::fs::read_to_string(vendor.join("LFLists/0TCG.lflist.conf"))?),
    );
    let threads = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let elfnote = (elfnote != 0).then(|| Deck::parse(include_str!("../../diff/elfnote-ycs-montreal-2026.deck"), cards));
    let deck = |rng: &mut StdRng| match &elfnote {
        Some(deck) => deck.shuffled(rng),
        None => pool.random_deck(rng),
    };

    let started = Instant::now();
    let reports: Vec<Report> = threads.install(|| {
        (seed..seed + duels)
            .into_par_iter()
            .map(|seed| play_random_duel(cards, &scripts, deck, seed, MAX_DECISIONS, |_| {}))
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
    if elfnote.is_some() {
        // Wall time without setup, assuming setup and play share the threads in proportion to their total times.
        let play: Duration = reports.iter().map(|report| report.play).sum();
        let oracle = decisions as f64 / (elapsed * play.as_secs_f64() / (play + setup).as_secs_f64());
        let replays: Vec<(Duel, &Report)> = (seed..)
            .zip(&reports)
            .map(|(seed, report)| Ok((engine_duel(seed, cards, deck)?, report)))
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

// A started engine duel with the decks and duel seed play_random_duel draws for `seed`.
fn engine_duel(seed: u64, cards: &'static CardDatabase, deck: impl Fn(&mut StdRng) -> Deck) -> Result<Duel, engine::DuelError> {
    let mut rng = StdRng::seed_from_u64(seed);
    let player = PlayerOptions::default();
    let mut duel = Duel::new(DuelOptions {
        seed: rng.random(),
        flags: DUEL_MODE_MR5,
        team1: player,
        team2: player,
    })?;
    duel.load_library(&scripts::LIBRARY);
    for team in 0..2 {
        duel.add_deck(team, &deck(&mut rng), cards)?;
    }
    duel.start();
    Ok(duel)
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
