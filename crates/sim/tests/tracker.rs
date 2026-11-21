mod common;

use std::io::Write;
use std::sync::OnceLock;

use common::{Result, Tracked, check_truth, follow_swapped};
use diff::all_records;
use engine::Prompt;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rayon::prelude::*;
use scripts::CARD_SCRIPTS;
use sim::{Vocabulary, features};

// Hidden cards are swapped at every STEP-th idle prompt.
const STEP: usize = 25;

fn vocabulary() -> &'static Vocabulary {
    static VOCABULARY: OnceLock<Vocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(|| Vocabulary::new(CARD_SCRIPTS.iter().map(|card| card.code)))
}

fn bag(tracked: &Tracked, player: u8) -> Vec<u32> {
    let mut bag = Vec::new();
    features(&tracked.duel, tracked.tracker(player), vocabulary(), &mut bag);
    bag
}

// A bag in range and sorted, the same again and for a copy of the duel; returns its size.
fn check_features(tracked: &Tracked, player: u8) -> Result<usize> {
    let encoded = bag(tracked, player);
    if encoded.iter().any(|&index| index as usize >= vocabulary().features()) || !encoded.is_sorted() {
        return Err(format!("player {player}'s features out of range or order: {encoded:?}").into());
    }
    if encoded != bag(tracked, player) || encoded != bag(&tracked.clone(), player) {
        return Err(format!("player {player}'s features differ when encoded again").into());
    }
    Ok(encoded.len())
}

#[test]
fn should_claim_only_what_is_true_and_encode_in_range() {
    let records = all_records().unwrap();
    let checked = records.par_iter().enumerate().map(|(index, (vendor, record))| -> Result<[usize; 4]> {
        let mut tracked = Tracked::new(vendor, record)?;
        let [mut states, mut known, mut features] = [0; 3];
        for (at, &action) in record.actions.iter().enumerate() {
            let context = |error: Box<dyn std::error::Error + Send + Sync>| format!("record {index}, action {at}: {error}");
            for player in 0..2 {
                known += check_truth(&tracked.duel, tracked.tracker(player), &record.position).map_err(context)?;
                features += check_features(&tracked, player).map_err(context)?;
            }
            states += 2;
            tracked.apply(action)?;
        }
        let drift = tracked.trackers.iter().map(|tracker| tracker.drift() as usize).sum();
        Ok([states, known, features, drift])
    });
    let checked: Vec<[usize; 4]> = checked.collect::<Result<_>>().unwrap();
    let drifted: Vec<usize> = (checked.iter().enumerate())
        .filter_map(|(index, &[.., drift])| (drift > 0).then_some(index))
        .collect();
    let sum = |[a, b, c, d]: [usize; 4], [e, f, g, h]: [usize; 4]| [a + e, b + f, c + g, d + h];
    let [states, known, features, drift] = checked.into_iter().fold([0; 4], sum);
    writeln!(
        std::io::stdout(),
        "{states} states: {:.2} of the opponent's hidden cards known, {:.1} features, {drift} drifts",
        known as f64 / states as f64,
        features as f64 / states as f64
    )
    .unwrap();
    assert!(drifted.is_empty(), "drifts in records {drifted:?}");
}

#[test]
fn should_track_the_same_in_duels_that_differ_only_in_what_the_player_never_saw() {
    let records = all_records().unwrap();
    let checked = records.par_iter().enumerate().map(|(index, (vendor, record))| -> Result<usize> {
        let mut rng = StdRng::seed_from_u64(index as u64);
        let mut tracked = Tracked::new(vendor, record)?;
        let mut compared = 0;
        for (at, &action) in record.actions.iter().enumerate() {
            if at % STEP == 0 && matches!(tracked.duel.prompt(), Some(Prompt::IdleCommand { .. })) {
                let rest = record.actions.get(at..).unwrap_or_default();
                for player in 0..2 {
                    compared += follow_swapped(&tracked, player, rest, &mut rng, |original, copy| {
                        let same = original.tracker(player) == copy.tracker(player) && bag(original, player) == bag(copy, player);
                        same.then_some(()).ok_or("tells duels apart that differ in what it never saw".into())
                    })?;
                }
            }
            tracked.apply(action)?;
        }
        Ok(compared)
    });
    let compared: usize = checked.collect::<Result<Vec<_>>>().unwrap().into_iter().sum();
    writeln!(std::io::stdout(), "{compared} states compared after a swap of hidden cards").unwrap();
    assert!(compared > 1000, "{compared} states compared");
}
