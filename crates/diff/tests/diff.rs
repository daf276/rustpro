use std::io::Write;
use std::ops::Range;
use std::path::{Path, PathBuf};

use diff::{DiffError, Report, SCENARIOS, SCENARIOS_FULL, Tally, Vendor, diff_cards, diff_duel, summarize};
use rayon::prelude::*;
use scripts::{CARD_SCRIPTS, CLASSIC_CARDS};

const WIN_BY_LP: u8 = 1;
const WIN_BY_DECK_OUT: u8 = 2;

fn load(vendor: fn(&Path) -> Result<Vendor, DiffError>) -> Result<&'static Vendor, DiffError> {
    Ok(Box::leak(Box::new(vendor(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor"),
    )?)))
}

fn diff_duels(vendor: &'static Vendor, seeds: Range<u64>) -> Result<(Tally, Vec<Report>), DiffError> {
    let results: Vec<Result<Report, DiffError>> = seeds.into_par_iter().map(|seed| diff_duel(vendor, seed)).collect();
    let failures: Vec<String> = results
        .iter()
        .filter_map(|result| result.as_ref().err())
        .map(ToString::to_string)
        .take(3)
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let reports: Vec<Report> = results.into_iter().flatten().collect();
    let tally = reports.iter().fold(Tally::default(), |tally, report| tally + report.tally.clone());
    Ok((tally, reports))
}

// Writes the resolved chain links per card and effect description (shown with --nocapture).
fn write_tally(tally: &Tally) {
    let out = &mut std::io::stdout().lock();
    for (&(code, description), count) in &tally.effects {
        writeln!(out, "{code} {description:#x}: {count}").ok();
    }
    writeln!(out, "chain links disabled: {}, replays: {}", tally.chain_disabled, tally.replays).ok();
    writeln!(out, "sum selections (exact, reaching): {:?}", tally.sum_prompts).ok();
}

fn assert_normal_monsters_clean(seeds: Range<u64>) -> Result<(), DiffError> {
    let (tally, reports) = diff_duels(load(Vendor::normal_monsters)?, seeds)?;
    let reasons: Vec<u8> = reports.iter().map(|report| report.reason).collect();
    // Replays stay at 0: without effects nothing changes the opponent's monsters while an attack is pending.
    let exercised = [
        tally.summons,
        tally.sets,
        tally.flip_summons,
        tally.position_changes,
        tally.tributes,
        tally.unselect_prompts,
        tally.attacks,
        tally.direct_attacks,
        tally.battle_destructions,
    ];
    assert!(exercised.iter().all(|&count| count > 0), "{tally:?}");
    assert!(
        [WIN_BY_LP, WIN_BY_DECK_OUT].iter().all(|reason| reasons.contains(reason)),
        "{reasons:?}"
    );
    Ok(())
}

fn assert_classic_effects_clean(seeds: Range<u64>) -> Result<(), DiffError> {
    let (tally, _) = diff_duels(load(Vendor::classic_effects)?, seeds)?;
    let resolutions: Vec<(u32, u32)> = CLASSIC_CARDS.iter().map(|&code| (code, tally.resolutions(code))).collect();
    assert!(
        resolutions.iter().all(|&(_, count)| count > 0) && tally.chain_disabled > 0,
        "{resolutions:?} {tally:?}"
    );
    Ok(())
}

#[test]
fn should_match_the_oracle_message_for_message_in_normal_monster_duels() {
    assert_normal_monsters_clean(0..1000).unwrap();
}

#[test]
#[ignore = "phase 1 exit, run with --release -- --ignored"]
fn should_match_the_oracle_in_5000_normal_monster_duels() {
    assert_normal_monsters_clean(0..5000).unwrap();
}

#[test]
fn should_match_the_oracle_message_for_message_in_classic_effect_duels() {
    assert_classic_effects_clean(0..1000).unwrap();
}

#[test]
#[ignore = "phase 2a exit, run with --release -- --ignored"]
fn should_match_the_oracle_in_5000_classic_effect_duels() {
    assert_classic_effects_clean(0..5000).unwrap();
}

// Checks that every card of the main and Extra Deck resolves some effect.
fn assert_elfnote_clean(seeds: Range<u64>) -> Result<(), DiffError> {
    let vendor = load(Vendor::elfnote)?;
    let (tally, _) = diff_duels(vendor, seeds)?;
    write_tally(&tally);
    let cards = vendor.deck.iter().flat_map(|deck| deck.main.iter().chain(&deck.extra));
    let unresolved: Vec<u32> = cards.copied().filter(|&code| tally.resolutions(code) == 0).collect();
    assert!(unresolved.is_empty(), "{unresolved:?} {tally:?}");
    Ok(())
}

#[test]
fn should_match_the_oracle_message_for_message_in_elfnote_mirror_matches() {
    assert_elfnote_clean(0..1000).unwrap();
}

#[test]
#[ignore = "phase 2c exit, run with --release -- --ignored"]
fn should_match_the_oracle_in_5000_elfnote_mirror_matches() {
    assert_elfnote_clean(0..5000).unwrap();
}

// Writes each card's scenarios out and checks that they diff clean and pass the card's checks, and that Ritual
// materials are selected by both kinds of sum.
fn assert_scenarios_clean(count: u32) -> Result<(), DiffError> {
    let vendor = load(Vendor::scenarios)?;
    let cards: Vec<u32> = CARD_SCRIPTS.iter().map(|card| card.code).collect();
    let runs = diff_cards(vendor, &cards, count);
    let out = &mut std::io::stdout().lock();
    let mut failed = Vec::new();
    for (&card, results) in cards.iter().zip(&runs) {
        let (passed, summary) = summarize(card, results);
        write!(out, "{summary}").ok();
        if !passed {
            failed.push(card);
        }
    }
    let reports = runs.iter().flatten().flatten();
    let tally = reports.fold(Tally::default(), |tally, report| tally + report.tally.clone());
    writeln!(out, "sum selections (exact, reaching): {:?}", tally.sum_prompts).ok();
    assert!(
        failed.is_empty() && tally.sum_prompts.iter().all(|&count| count > 0),
        "failing cards {failed:?}, sum selections {:?}",
        tally.sum_prompts
    );
    Ok(())
}

#[test]
fn should_match_the_oracle_message_for_message_in_per_card_scenarios() {
    assert_scenarios_clean(SCENARIOS).unwrap();
}

#[test]
#[ignore = "phase 4a, run with --release -- --ignored"]
fn should_match_the_oracle_in_64_scenarios_per_card() {
    assert_scenarios_clean(SCENARIOS_FULL).unwrap();
}
