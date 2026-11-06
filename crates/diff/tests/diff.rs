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
// needs 1000 or more battle damage from it.
const SUMMONED_ONLY: [u32; 5] = [91949988, 39317553, 1641882, 96334243, 26050548];
const LOCATION_HAND: u8 = 0x2;
const LOCATION_MZONE: u8 = 0x4;
// Materials whose use exercises a procedure path of phase 3d, by card code and the location they leave: Mono Synchron
// (EFFECT_SYNCHRO_MATERIAL_CUSTOM and EFFECT_SYNCHRO_MAT_RESTRICTION) and Revolution Synchron from the hand
// (EFFECT_SYNCHRO_MAT_FROM_HAND).
const MATERIAL_PATHS: [(u32, u8); 2] = [(56897896, LOCATION_MZONE), (97682931, LOCATION_HAND)];
// Hints the core adds to a card when a procedure path of phase 3d runs, by the carrier and the description: Gemini
// Lancer's second Normal Summon (64).
const HINT_PATHS: [(u32, u64); 1] = [(26254876, 64)];
// Messages only a procedure path of phase 3d writes, by the carrier and the message type: Fiendish Chain's card target
// (MSG_CARD_TARGET), Demotion's and Salamandra's equips (MSG_EQUIP).
const MESSAGE_PATHS: [(u32, u8); 3] = [(50078509, 96), (72575145, 93), (32268901, 93)];
// Normal Summons and Sets by a procedure, by the carrier and the number of tributes: Blizzard Princess and Darklord
// Desire with 1.
const SUMMON_PATHS: [(u32, u32); 2] = [(28348537, 1), (55690251, 1)];
const SET_PATHS: [(u32, u32); 1] = [(55690251, 1)];

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

// Writes per card under test how many of its scenarios resolved one of its effects and how often it was Special
// Summoned, and checks that every ported card resolves some effect, including the effects only scenarios reach.
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
        writeln!(
            out,
            "{card}: {resolving} of {} scenarios resolve its effects, {summons} Special Summons",
            own.count()
        )
        .ok();
    }
    let materials = MATERIAL_PATHS.map(|key| (key, tally.materials.get(&key).copied().unwrap_or(0)));
    writeln!(out, "material paths: {materials:?}").ok();
    let hints = HINT_PATHS.map(|(card, description)| ((card, description), tally.card_hints.get(&description).copied().unwrap_or(0)));
    let messages = MESSAGE_PATHS.map(|(card, kind)| ((card, kind), tally.messages.get(&kind).copied().unwrap_or(0)));
    writeln!(out, "message paths: {messages:?}").ok();
    let summons = SUMMON_PATHS.map(|key| (key, tally.summon_tributes.get(&key).copied().unwrap_or(0)));
    writeln!(out, "summon paths: {summons:?}").ok();
    let sets = SET_PATHS.map(|key| (key, tally.set_tributes.get(&key).copied().unwrap_or(0)));
    writeln!(out, "set paths: {sets:?}").ok();
    let paths: Vec<(u32, u32)> = (materials.iter().map(|&((card, _), count)| (card, count)))
        .chain(hints.iter().map(|&((card, _), count)| (card, count)))
        .chain(messages.iter().map(|&((card, _), count)| (card, count)))
        .chain(summons.iter().map(|&((card, _), count)| (card, count)))
        .chain(sets.iter().map(|&((card, _), count)| (card, count)))
        .collect();
    let exercised = |code: u32| paths.iter().any(|&(card, count)| card == code && count > 0);
    writeln!(out, "hint paths: {hints:?}").ok();
    let unresolved: Vec<u32> = vendor
        .scenario_cards
        .iter()
        .copied()
        .filter(|&code| match SUMMONED_ONLY.contains(&code) {
            true => !tally.special_summons.contains_key(&code),
            false => tally.resolutions(code) == 0 && !exercised(code),
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
        unresolved.is_empty() && only_in_scenarios.iter().all(Option::is_some) && paths.iter().all(|&(_, count)| count > 0),
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
