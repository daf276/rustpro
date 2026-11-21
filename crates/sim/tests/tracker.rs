use std::collections::BTreeMap;
use std::io::Write;
use std::sync::OnceLock;

use diff::{Vendor, all_records};
use engine::constants::*;
use engine::{Action, CardId, Duel, DuelRecord, Position, Prompt};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use scripts::{CARD_SCRIPTS, LIBRARY};
use sim::{Tracker, Vocabulary, features, mask};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Hidden cards are swapped at every STEP-th idle prompt and the record followed for up to FOLLOWED actions from there.
const STEP: usize = 25;
const FOLLOWED: usize = 60;
const PILES: [u8; 5] = [LOCATION_DECK, LOCATION_HAND, LOCATION_GRAVE, LOCATION_REMOVED, LOCATION_EXTRA];

fn vocabulary() -> &'static Vocabulary {
    static VOCABULARY: OnceLock<Vocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(|| Vocabulary::new(CARD_SCRIPTS.iter().map(|card| card.code)))
}

// A record's duel with a tracker per player, updated after every action.
#[derive(Clone)]
struct Tracked {
    duel: Duel,
    trackers: [Tracker; 2],
}

impl Tracked {
    fn new(vendor: &'static Vendor, record: &DuelRecord) -> Result<Self> {
        let mut duel = Duel::from_position(&record.position, &vendor.cards, &LIBRARY, record.seed)?;
        duel.set_trace(true);
        let trackers = [0, 1].map(|player| Tracker::new(&duel, player));
        let mut tracked = Self { duel, trackers };
        tracked.duel.process();
        tracked.update();
        Ok(tracked)
    }

    // Returns the unmasked messages of the action.
    fn apply(&mut self, action: Action) -> Result<Vec<u8>> {
        self.duel.apply(action)?;
        Ok(self.update())
    }

    fn update(&mut self) -> Vec<u8> {
        let messages = self.duel.take_trace();
        self.trackers.iter_mut().for_each(|tracker| tracker.update(&messages, &self.duel));
        messages
    }

    fn tracker(&self, player: u8) -> &Tracker {
        let [first, second] = &self.trackers;
        if player == 0 { first } else { second }
    }

    fn features(&self, player: u8) -> Vec<u32> {
        let mut bag = Vec::new();
        features(&self.duel, self.tracker(player), vocabulary(), &mut bag);
        bag
    }
}

// Every code the tracker claims is the card's, its unplaced codes are among the codes of the pile's cards it does not know, and
// the opponent started with as many copies of each code shown; returns how many of the opponent's hidden cards it knows.
fn check_truth(duel: &Duel, tracker: &Tracker, start: &Position) -> Result<usize> {
    let player = tracker.player();
    let observation = duel.observe(player);
    let position = duel.position();
    let mut unknown: BTreeMap<(u8, u8), Vec<u32>> = BTreeMap::new();
    let mut known = 0;
    for (seen, placed) in observation.cards.iter().zip(&position.cards) {
        let (claimed, code) = (tracker.code_at(seen), placed.info.code);
        if claimed != 0 && claimed != code {
            return Err(format!("player {player} takes {code} at {seen:?} for {claimed}").into());
        }
        known += usize::from(claimed != 0 && seen.code == 0 && seen.controller != player);
        if claimed == 0 {
            unknown.entry((seen.controller, seen.location)).or_default().push(code);
        }
    }
    for (controller, location) in (0..2).flat_map(|controller| PILES.map(|location| (controller, location))) {
        let mut codes = unknown.remove(&(controller, location)).unwrap_or_default();
        for &code in tracker.unplaced(controller, location) {
            let at = codes.iter().position(|&other| other == code);
            let at =
                at.ok_or_else(|| format!("player {player} takes {code} for an unknown card of {location:#x} of player {controller}"))?;
            codes.swap_remove(at);
            known += usize::from(controller != player);
        }
    }
    for (code, count) in tracker.shown() {
        let owned = (start.cards.iter())
            .filter(|card| (card.info.team, card.info.code) == (1 - player, code))
            .count();
        if count as usize > owned {
            return Err(format!("player {player} was shown {count} copies of {code}, the opponent has {owned}").into());
        }
    }
    Ok(known)
}

// A bag in range and sorted, the same again and for a copy of the duel; returns its size.
fn check_features(tracked: &Tracked, player: u8) -> Result<usize> {
    let bag = tracked.features(player);
    if bag.iter().any(|&index| index as usize >= vocabulary().features()) || !bag.is_sorted() {
        return Err(format!("player {player}'s features out of range or order: {bag:?}").into());
    }
    if bag != tracked.features(player) || bag != tracked.clone().features(player) {
        return Err(format!("player {player}'s features differ when encoded again").into());
    }
    Ok(bag.len())
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

// The opponent's hand and Deck cards hidden from `player`, as the copy of `duel` that swaps them in pairs of different codes, if any.
fn swapped(duel: &Duel, player: u8, rng: &mut StdRng) -> Option<Duel> {
    let opponent = 1 - player;
    let observation = duel.observe(player);
    let hidden = |card: &CardId| {
        let place = (card.get_location(duel), card.get_sequence(duel));
        let mut cards = observation
            .cards
            .iter()
            .filter(|seen| seen.controller == opponent && seen.code == 0);
        cards.any(|seen| (u32::from(seen.location), u32::from(seen.sequence)) == place)
    };
    let group = duel.get_field_group(opponent, u32::from(LOCATION_DECK | LOCATION_HAND), 0);
    let mut cards: Vec<CardId> = group.iter().filter(hidden).collect();
    cards.shuffle(rng);
    let (pairs, _) = cards.as_chunks::<2>();
    let pairs: Vec<[CardId; 2]> = (pairs.iter().copied())
        .filter(|[card, other]| card.get_code(duel) != other.get_code(duel))
        .collect();
    let mut copy = duel.clone();
    pairs.iter().for_each(|&[card, other]| copy.swap_cards(card, other));
    (!pairs.is_empty()).then_some(copy)
}

// From a copy whose opponent's hidden cards are swapped, plays on both while `player`'s masked messages and observations agree,
// its trackers and features must agree too; returns the states compared.
fn follow_swapped(tracked: &Tracked, player: u8, actions: &[Action], rng: &mut StdRng) -> Result<usize> {
    let Some(duel) = swapped(&tracked.duel, player, rng) else {
        return Ok(0);
    };
    let mut copies = [tracked.clone(), Tracked { duel, ..tracked.clone() }];
    let mut compared = 0;
    for &action in actions.iter().take(FOLLOWED) {
        let [original, copy] = &mut copies;
        let messages = original.apply(action)?;
        let Ok(other) = copy.apply(action) else { break };
        let [masked, other] = [messages, other].map(|messages| {
            let mut masked = Vec::new();
            mask(&messages, player, &mut masked);
            masked
        });
        if masked != other || original.duel.observe(player) != copy.duel.observe(player) {
            break;
        }
        if original.tracker(player) != copy.tracker(player) || original.features(player) != copy.features(player) {
            return Err(format!("player {player} tells duels apart that differ in what it never saw, after {action:?}").into());
        }
        compared += 1;
    }
    Ok(compared)
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
                    compared += follow_swapped(&tracked, player, rest, &mut rng)?;
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
