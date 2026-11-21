use std::collections::BTreeMap;

use diff::Vendor;
use engine::constants::*;
use engine::{Action, CardId, Duel, DuelRecord, Position};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use scripts::LIBRARY;
use sim::{Tracker, mask};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// A swapped copy is followed for up to FOLLOWED actions.
const FOLLOWED: usize = 60;
const PILES: [u8; 5] = [LOCATION_DECK, LOCATION_HAND, LOCATION_GRAVE, LOCATION_REMOVED, LOCATION_EXTRA];

// A record's duel with a tracker per player, updated after every action.
#[derive(Clone)]
pub struct Tracked {
    pub duel: Duel,
    pub trackers: [Tracker; 2],
}

impl Tracked {
    pub fn new(vendor: &'static Vendor, record: &DuelRecord) -> Result<Self> {
        let mut duel = Duel::from_position(&record.position, &vendor.cards, &LIBRARY, record.seed)?;
        duel.set_trace(true);
        let trackers = [0, 1].map(|player| Tracker::new(&duel, player));
        let mut tracked = Self { duel, trackers };
        tracked.duel.process();
        tracked.update();
        Ok(tracked)
    }

    // Returns the unmasked messages of the action.
    pub fn apply(&mut self, action: Action) -> Result<Vec<u8>> {
        self.duel.apply(action)?;
        Ok(self.update())
    }

    fn update(&mut self) -> Vec<u8> {
        let messages = self.duel.take_trace();
        self.trackers.iter_mut().for_each(|tracker| tracker.update(&messages, &self.duel));
        messages
    }

    pub fn tracker(&self, player: u8) -> &Tracker {
        let [first, second] = &self.trackers;
        if player == 0 { first } else { second }
    }
}

// Every code the tracker claims is the card's, its unplaced codes are among the codes of the pile's cards it does not know, and
// the opponent started with as many copies of each code shown; returns how many of the opponent's hidden cards it knows.
pub fn check_truth(duel: &Duel, tracker: &Tracker, start: &Position) -> Result<usize> {
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
// and `check`s that `player` cannot tell them apart; returns the states compared.
pub fn follow_swapped(
    tracked: &Tracked,
    player: u8,
    actions: &[Action],
    rng: &mut StdRng,
    check: impl Fn(&Tracked, &Tracked) -> Result,
) -> Result<usize> {
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
        check(original, copy).map_err(|error| format!("player {player}, after {action:?}: {error}"))?;
        compared += 1;
    }
    Ok(compared)
}
