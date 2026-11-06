use std::io::Write;
use std::ops::Range;
use std::path::{Path, PathBuf};

use diff::{DiffError, Report, Tally, Vendor, diff_duel};
use rayon::prelude::*;
use scripts::CLASSIC_CARDS;

const WIN_BY_LP: u8 = 1;
const WIN_BY_DECK_OUT: u8 = 2;
// The card code and description (aux.Stringid(code, 0)) of effects that random mirror matches never resolve.
const ECCLESIA_DARK_DRAGON_SUMMON: (u32, u64) = (78397661, 78397661 << 20);
const DAWN_DRAGSTER_LEVEL_UP: (u32, u64) = (33158448, 33158448 << 20);
const VIDOLIUM_BANISH: (u32, u64) = (70488851, (70488851 << 20) + 3);
const ALBION_FUSION_SUMMON: (u32, u64) = (87746184, 87746184 << 20);
// Ported cards whose effects scenarios do not resolve, checked by their Special Summons instead: Gaia Dragon, the
// Thunder Charger has only a continuous effect, Drill Driver Vespenato's trigger needs it Xyz Summoned and then
// destroyed by an opponent's card effect, Fusionist and Sea Monster of Theseus have no effect, and Extox Hydra's draw
// needs 1000 or more battle damage from it; Magician of Black Chaos has no effect.
const SUMMONED_ONLY: [u32; 6] = [91949988, 39317553, 1641882, 96334243, 26050548, 30208479];
// Ritual Raven has only its Ritual level, checked by its use as a Ritual Summon material.
const RITUAL_MATERIAL_ONLY: [u32; 1] = [34334692];

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

// Writes per card under test how many of its scenarios resolved one of its effects, how often it was Special Summoned
// and used as a Ritual material, and checks that every ported card resolves some effect, including the effects only
// scenarios reach, and that Ritual materials are selected by both kinds of sum.
fn assert_scenarios_clean(seeds: Range<u64>) -> Result<(), DiffError> {
    let vendor = load(Vendor::scenarios)?;
    let (tally, reports) = diff_duels(vendor, seeds.clone())?;
    write_tally(&tally);
    let out = &mut std::io::stdout().lock();
    for &card in &vendor.scenario_cards {
        let own = seeds
            .clone()
            .zip(&reports)
            .filter(|&(seed, _)| vendor.scenario_card(seed) == Some(card));
        let resolving = own.clone().filter(|(_, report)| report.tally.resolutions(card) > 0).count();
        let summons = tally.special_summons.get(&card).copied().unwrap_or(0);
        let materials = tally.ritual_materials.get(&card).copied().unwrap_or(0);
        writeln!(
            out,
            "{card}: {resolving} of {} scenarios resolve its effects, {summons} Special Summons, {materials} Ritual materials",
            own.count()
        )
        .ok();
    }
    let unresolved: Vec<u32> = vendor
        .scenario_cards
        .iter()
        .copied()
        .filter(|code| match (SUMMONED_ONLY.contains(code), RITUAL_MATERIAL_ONLY.contains(code)) {
            (true, _) => !tally.special_summons.contains_key(code),
            (_, true) => !tally.ritual_materials.contains_key(code),
            _ => tally.resolutions(*code) == 0,
        })
        .collect();
    let only_in_scenarios = [
        ECCLESIA_DARK_DRAGON_SUMMON,
        DAWN_DRAGSTER_LEVEL_UP,
        VIDOLIUM_BANISH,
        ALBION_FUSION_SUMMON,
    ]
    .map(|effect| tally.effects.get(&effect));
    assert!(
        unresolved.is_empty() && only_in_scenarios.iter().all(Option::is_some) && tally.sum_prompts.iter().all(|&count| count > 0),
        "{unresolved:?} {tally:?}"
    );
    Ok(())
}

#[test]
fn should_match_the_oracle_message_for_message_in_per_card_scenarios() {
    assert_scenarios_clean(0..1000).unwrap();
}

#[test]
#[ignore = "phase 2d exit, run with --release -- --ignored"]
fn should_match_the_oracle_in_5000_per_card_scenarios() {
    assert_scenarios_clean(0..5000).unwrap();
}
