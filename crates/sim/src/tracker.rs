// What a player knows of a duel and remembers, read only from the messages EDOPro's server sends it ([mask]) and what
// [Duel::observe] shows it, so it holds nothing the player could not know. The rules:
// - A pile (Deck, hand, GY, banished, Extra Deck) keeps the code of each place it knows, and the codes known to be among
//   its cards of unknown code ("unplaced"); a shuffle the player does not see moves all its codes there.
// - A card leaving a pile from a place of unknown code takes one copy of its shown code off the unplaced codes, or if unseen
//   one copy of each code, as each could have been it; the codes stay in [Tracker::shown].
// - A pile also keeps which of its places hold the other player's cards (an opponent's card added to a hand); a shuffle moves
//   their codes among the pile's other cards of that player, and a card of such a code found in the pile is taken for one; a
//   card leaving it unseen while it holds such codes has an unknown owner.
// - Field zones keep their card's code and owner, so a card followed from a known place stays known face-down.
// - [Tracker::shown]: per code of the opponent's cards, the most copies known at once, or 1 once seen.
use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};

use engine::constants::*;
use engine::{Duel, Memory, Observation, SeenCard};

use crate::mask::{frames, mask, u32_at};

const PILES: [u8; 5] = [LOCATION_DECK, LOCATION_HAND, LOCATION_GRAVE, LOCATION_REMOVED, LOCATION_EXTRA];
const ZONES: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Pile {
    places: Vec<Slot>,
    unplaced: Vec<u32>,
    // The codes among `unplaced` of the other player's cards.
    foreign: Vec<u32>,
}

// A pile's place: the code of its card (0 unknown) and whether the card is the other player's.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Slot {
    code: u32,
    foreign: bool,
}

// A card as it moves: its code (0 unknown), owner and, on the field, its Xyz materials.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Held {
    code: u32,
    owner: Option<u8>,
    materials: Vec<Held>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Side {
    // In the order of [PILES].
    piles: [Pile; PILES.len()],
    // Monster Zones, then Spell & Trap Zones.
    zones: [[Option<Held>; ZONES]; 2],
    // This turn's Normal Summons and Sets, and the codes of the cards whose effects the player activated; counted from the
    // messages, as the engine's summon count and count limits can depend on which hidden card was Set.
    summons: u32,
    activated: Vec<u32>,
}

// Per controller, indexed by 0 or 1.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Sides([Side; 2]);

impl Index<u8> for Sides {
    type Output = Side;

    fn index(&self, controller: u8) -> &Side {
        let [first, second] = &self.0;
        if controller & 1 == 0 { first } else { second }
    }
}

impl IndexMut<u8> for Sides {
    fn index_mut(&mut self, controller: u8) -> &mut Side {
        let [first, second] = &mut self.0;
        if controller & 1 == 0 { first } else { second }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Place {
    controller: u8,
    location: u8,
    sequence: usize,
    // The position, or for an Xyz material its index among the materials.
    position: u32,
}

impl Place {
    // A loc_info: controller, location, sequence and position.
    fn read(message: &[u8], at: usize) -> Self {
        let byte = |offset| message.get(at + offset).copied().unwrap_or(0);
        Self {
            controller: byte(0) & 1,
            location: byte(1),
            sequence: u32_at(message, at + 2) as usize,
            position: u32_at(message, at + 6),
        }
    }

    fn seen(card: &SeenCard) -> Self {
        Self {
            controller: card.controller & 1,
            location: card.location,
            sequence: card.sequence.into(),
            position: card.position.into(),
        }
    }

    fn same_zone(&self, other: &Self) -> bool {
        (self.controller, self.location, self.sequence) == (other.controller, other.location, other.sequence)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tracker {
    player: u8,
    sides: Sides,
    shown: BTreeMap<u32, u32>,
    // A field card a move put another in place of, until the move of it that follows.
    displaced: Option<(Place, Held)>,
    // Materials attached to a card off the field (an Xyz Monster in the Extra Deck while it is summoned), until it moves.
    staged: Vec<(Place, Held)>,
    drift: u32,
}

fn pile_index(location: u8) -> Option<usize> {
    PILES.iter().position(|&pile| pile == location)
}

fn zone_index(location: u8) -> Option<usize> {
    match location & !LOCATION_OVERLAY {
        LOCATION_MZONE => Some(0),
        LOCATION_SZONE => Some(1),
        _ => None,
    }
}

fn remove_one(codes: &mut Vec<u32>, code: u32) -> bool {
    let at = codes.iter().position(|&other| other == code);
    at.map(|at| codes.swap_remove(at)).is_some()
}

impl Pile {
    // Takes the card at `sequence`, `code` as the message shows it; returns its code and whether it is the other player's, if
    // known.
    fn take(&mut self, sequence: usize, code: u32) -> (u32, Option<bool>) {
        let known = if sequence < self.places.len() { self.places.remove(sequence) } else { Slot::default() };
        match (known.code, code) {
            (0, 0) => {
                self.unplaced.sort_unstable();
                let mut previous = None;
                self.unplaced.retain(|&code| previous.replace(code) == Some(code));
                (0, (known.foreign || self.foreign.is_empty()).then_some(known.foreign))
            }
            (0, code) => {
                remove_one(&mut self.unplaced, code);
                (code, Some(known.foreign || remove_one(&mut self.foreign, code)))
            }
            (known_code, _) => (if code != 0 { code } else { known_code }, Some(known.foreign)),
        }
    }

    fn take_top(&mut self, code: u32) -> u32 {
        self.take(self.places.len().saturating_sub(1), code).0
    }

    // Returns whether the card is the other player's.
    fn reveal(&mut self, sequence: usize, code: u32) -> bool {
        let Some(slot) = self.places.get_mut(sequence) else { return false };
        if slot.code == 0 {
            remove_one(&mut self.unplaced, code);
            slot.foreign = slot.foreign || remove_one(&mut self.foreign, code);
        }
        slot.code = code;
        slot.foreign
    }

    fn shuffle(&mut self) {
        for slot in self.places.iter().filter(|slot| slot.code != 0) {
            self.unplaced.push(slot.code);
            if slot.foreign {
                self.foreign.push(slot.code);
            }
        }
        self.places.fill(Slot::default());
    }
}

impl Side {
    fn pile(&mut self, location: u8) -> Option<&mut Pile> {
        self.piles.get_mut(pile_index(location)?)
    }

    fn zone(&mut self, location: u8, sequence: usize) -> Option<&mut Option<Held>> {
        self.zones.get_mut(zone_index(location)?)?.get_mut(sequence)
    }
}

impl Tracker {
    // From what the duel shows `player` now and its own Deck, which it knows as the deck it brought.
    pub fn new(duel: &Duel, player: u8) -> Self {
        let observation = duel.observe(player);
        let position = duel.position();
        let own = (observation.cards.iter().zip(&position.cards))
            .filter(|(seen, _)| seen.controller == player && seen.location == LOCATION_DECK)
            .map(|(_, placed)| placed.info.code);
        let mut tracker = Self { player, ..Self::default() };
        tracker.sync(&observation);
        tracker.sides[player].piles[0].unplaced.extend(own);
        tracker.drift = 0;
        tracker
    }

    pub fn player(&self) -> u8 {
        self.player
    }

    // Reads `messages`, the duel's unmasked trace since the last update, through [mask], then what the duel shows now.
    pub fn update(&mut self, messages: &[u8], duel: &Duel) {
        let mut masked = Vec::with_capacity(messages.len());
        mask(messages, self.player, &mut masked);
        self.apply(&masked, &duel.observe(self.player));
    }

    fn apply(&mut self, masked: &[u8], observation: &Observation) {
        frames(masked).for_each(|message| self.read(message));
        self.displaced = None;
        self.sync(observation);
    }

    // The code the player knows for a card the observation shows, 0 if unknown.
    pub fn code_at(&self, card: &SeenCard) -> u32 {
        let place = Place::seen(card);
        let side = &self.sides[place.controller];
        if let Some(pile) = pile_index(place.location) {
            return side
                .piles
                .get(pile)
                .and_then(|pile| pile.places.get(place.sequence))
                .map_or(0, |slot| slot.code);
        }
        let held = zone_index(place.location).and_then(|zone| side.zones.get(zone)?.get(place.sequence)?.as_ref());
        let held = match place.location & LOCATION_OVERLAY {
            0 => held,
            _ => held.and_then(|held| held.materials.get(place.position as usize)),
        };
        held.map_or(0, |held| held.code)
    }

    // The codes known to be among a pile's cards of unknown code.
    pub fn unplaced(&self, controller: u8, location: u8) -> &[u32] {
        let pile = pile_index(location).and_then(|pile| self.sides[controller].piles.get(pile));
        pile.map_or(&[], |pile| &pile.unplaced)
    }

    // The opponent's cards shown this game: per code the most copies known at once.
    pub fn shown(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.shown.iter().map(|(&code, &count)| (code, count))
    }

    // What the player remembers of the cards `duel` hides from it, for [Duel::determinizer_with].
    pub fn memory(&self, duel: &Duel) -> Memory {
        let observation = duel.observe(self.player);
        let piles = (0..2).flat_map(|controller| PILES.map(|location| (controller, location)));
        let unplaced = piles.flat_map(|(controller, location)| {
            let codes = self.unplaced(controller, location).iter();
            codes.map(move |&code| (controller, location, code))
        });
        Memory {
            codes: observation.cards.iter().map(|card| self.code_at(card)).collect(),
            unplaced: unplaced.collect(),
            shown: self.shown().collect(),
        }
    }

    pub fn normal_summons(&self, controller: u8) -> u32 {
        self.sides[controller].summons
    }

    pub fn activated(&self, controller: u8) -> &[u32] {
        &self.sides[controller].activated
    }

    // How often what the duel showed disagreed with the tracked piles and zones, which were then reset to it.
    pub fn drift(&self) -> u32 {
        self.drift
    }

    fn note(&mut self, owner: Option<u8>, code: u32) {
        if code != 0 && owner == Some(1 - self.player) {
            self.shown.entry(code).or_insert(1);
        }
    }

    fn read(&mut self, message: &[u8]) {
        let Some((&id, body)) = message.split_first() else { return };
        let (code, player) = (u32_at(body, 0), body.first().map_or(0, |player| player & 1));
        let entries = |at: usize, size: usize| (at..).step_by(size).take(u32_at(body, 1) as usize);
        match id {
            MSG_MOVE => {
                let held = self.take(Place::read(body, 4), code);
                self.put(Place::read(body, 14), held);
            }
            MSG_SWAP => {
                let [first, second] = [Place::read(body, 4), Place::read(body, 18)];
                let [held, other] = [(first, code), (second, u32_at(body, 14))].map(|(place, code)| self.take(place, code));
                self.put(first, other);
                self.put(second, held);
            }
            MSG_DRAW => {
                for entry in entries(5, 8) {
                    let side = &mut self.sides[player];
                    let code = side.piles[0].take_top(u32_at(body, entry));
                    side.piles[1].places.push(Slot { code, foreign: false });
                    self.note(Some(player), code);
                }
            }
            MSG_SHUFFLE_DECK => self.sides[player].piles[0].shuffle(),
            MSG_SHUFFLE_HAND | MSG_SHUFFLE_EXTRA => {
                let location = if id == MSG_SHUFFLE_HAND { LOCATION_HAND } else { LOCATION_EXTRA };
                if let Some(pile) = self.sides[player].pile(location) {
                    pile.shuffle();
                }
                for (sequence, entry) in entries(5, 4).enumerate() {
                    let place = Place {
                        controller: player,
                        location,
                        sequence,
                        position: 0,
                    };
                    self.reveal(place, u32_at(body, entry));
                }
            }
            MSG_CONFIRM_CARDS | MSG_CONFIRM_DECKTOP => {
                for entry in entries(5, 10) {
                    let place = Place::read(body, entry + 4);
                    self.reveal(Place { position: 0, ..place }, u32_at(body, entry));
                }
            }
            MSG_DECK_TOP => {
                let size = self.sides[player].piles[0].places.len();
                let place = Place {
                    controller: player,
                    location: LOCATION_DECK,
                    sequence: size.saturating_sub(1 + u32_at(body, 1) as usize),
                    position: 0,
                };
                self.reveal(place, u32_at(body, 5));
            }
            MSG_POS_CHANGE => {
                let byte = |at: usize| body.get(at).copied().unwrap_or(0);
                let place = Place {
                    controller: byte(4) & 1,
                    location: byte(5),
                    sequence: byte(6).into(),
                    position: 0,
                };
                self.reveal(place, code);
            }
            MSG_SUMMONING | MSG_SET | MSG_SPSUMMONING | MSG_FLIPSUMMONING | MSG_CHAINING => {
                let place = Place::read(body, 4);
                self.reveal(place, code);
                let side = &mut self.sides[place.controller];
                match id {
                    MSG_SUMMONING => side.summons += 1,
                    MSG_SET if place.location == LOCATION_MZONE => side.summons += 1,
                    MSG_CHAINING => self.sides[body.get(14).copied().unwrap_or(0)].activated.push(code),
                    _ => {}
                }
            }
            MSG_NEW_TURN => self.sides.0.iter_mut().for_each(|side| {
                side.summons = 0;
                side.activated.clear();
            }),
            MSG_SHUFFLE_SET_CARD => {
                for entry in (2..).step_by(10).take(body.get(1).copied().unwrap_or(0).into()) {
                    let place = Place::read(body, entry);
                    if let Some(Some(held)) = self.sides[place.controller].zone(place.location, place.sequence) {
                        held.code = 0;
                    }
                }
            }
            MSG_SWAP_GRAVE_DECK => {
                let side = &mut self.sides[player];
                side.piles.swap(0, 2);
                let flags = body.get(9..).unwrap_or_default();
                let to_extra = |index: usize| flags.get(index / 8).is_some_and(|&byte| byte >> (index % 8) & 1 != 0);
                let mut index = 0;
                let extra: Vec<Slot> = (side.piles[0].places.extract_if(.., |_| {
                    index += 1;
                    to_extra(index - 1)
                }))
                .collect();
                let at = (u32_at(body, 1) as usize).min(side.piles[4].places.len());
                side.piles[4].places.splice(at..at, extra);
                side.piles[0].shuffle();
            }
            _ => {}
        }
    }

    fn take(&mut self, place: Place, code: u32) -> Held {
        let side = &mut self.sides[place.controller];
        if let Some(pile) = side.pile(place.location) {
            let staged = self.staged.extract_if(.., |(xyz, _)| {
                xyz.same_zone(&Place {
                    location: place.location | LOCATION_OVERLAY,
                    ..place
                })
            });
            let (code, foreign) = pile.take(place.sequence, code);
            return Held {
                code,
                owner: foreign.map(|foreign| place.controller ^ u8::from(foreign)),
                materials: staged.map(|(_, material)| material).collect(),
            };
        }
        let displaced = self.displaced.take_if(|(zone, _)| zone.same_zone(&place));
        let overlay = place.location & LOCATION_OVERLAY != 0;
        let held = match (displaced, side.zone(place.location, place.sequence)) {
            (Some((_, held)), _) => Some(held),
            (None, Some(Some(xyz))) if overlay => {
                let index = place.position as usize;
                (index < xyz.materials.len()).then(|| xyz.materials.remove(index))
            }
            (None, Some(slot)) if !overlay => slot.take(),
            (None, _) => (self.staged.iter().position(|(xyz, _)| xyz.same_zone(&place))).map(|at| self.staged.remove(at).1),
        };
        let held = held.unwrap_or_default();
        Held {
            code: if code != 0 { code } else { held.code },
            ..held
        }
    }

    fn put(&mut self, place: Place, mut held: Held) {
        let (owner, code) = (held.owner, held.code);
        let side = &mut self.sides[place.controller];
        if let Some(pile) = side.pile(place.location) {
            let owner = owner.unwrap_or(place.controller);
            let slot = Slot { code, foreign: owner != place.controller };
            pile.places.insert(place.sequence.min(pile.places.len()), slot);
            return self.note(Some(owner), code);
        }
        let overlay = place.location & LOCATION_OVERLAY != 0;
        match side.zone(place.location, place.sequence) {
            Some(Some(xyz)) if overlay => {
                held.materials.clear();
                xyz.materials.insert((place.position as usize).min(xyz.materials.len()), held);
            }
            Some(slot) if !overlay => {
                if place.location == LOCATION_SZONE {
                    held.materials.clear();
                }
                if let Some(occupant) = slot.replace(held) {
                    self.displaced = Some((place, occupant));
                }
            }
            _ if overlay => {
                held.materials.clear();
                self.staged.push((place, held));
            }
            _ => {}
        }
        self.note(owner, code);
    }

    fn reveal(&mut self, place: Place, code: u32) {
        if code == 0 {
            return;
        }
        let side = &mut self.sides[place.controller];
        if let Some(pile) = side.pile(place.location) {
            let owner = place.controller ^ u8::from(pile.reveal(place.sequence, code));
            return self.note(Some(owner), code);
        }
        let held = match side.zone(place.location, place.sequence) {
            Some(Some(xyz)) if place.location & LOCATION_OVERLAY != 0 => xyz.materials.get_mut(place.position as usize),
            Some(Some(held)) => Some(held),
            _ => None,
        };
        if let Some(held) = held {
            held.code = code;
            let owner = held.owner;
            self.note(owner, code);
        }
    }

    // Resets what disagrees with the observation (pile sizes, occupied zones, material counts), then takes its codes.
    fn sync(&mut self, observation: &Observation) {
        let mut shape = Sides::default();
        for card in &observation.cards {
            let place = Place::seen(card);
            let side = &mut shape[place.controller];
            if let Some(pile) = side.pile(place.location) {
                pile.places.push(Slot::default());
            } else if let Some(slot) = side.zone(place.location, place.sequence) {
                let held = slot.get_or_insert_default();
                if place.location & LOCATION_OVERLAY != 0 {
                    held.materials.push(Held::default());
                }
            }
        }
        for (controller, (side, shape)) in (0..).zip(self.sides.0.iter_mut().zip(shape.0)) {
            for (pile, shape) in side.piles.iter_mut().zip(shape.piles) {
                if pile.places.len() != shape.places.len() {
                    self.drift += 1;
                    *pile = shape;
                }
            }
            for (slot, shape) in side.zones.iter_mut().flatten().zip(shape.zones.into_iter().flatten()) {
                match (slot.as_mut(), shape) {
                    (Some(held), Some(shape)) if held.materials.len() != shape.materials.len() => {
                        self.drift += 1;
                        held.materials = shape.materials;
                    }
                    (Some(_), None) => {
                        self.drift += 1;
                        *slot = None;
                    }
                    (None, Some(shape)) => {
                        self.drift += 1;
                        *slot = Some(Held {
                            owner: Some(controller),
                            ..shape
                        });
                    }
                    _ => {}
                }
            }
        }
        for card in observation.cards.iter().filter(|card| card.code != 0) {
            self.reveal(Place::seen(card), card.code);
        }
        self.count_shown();
    }

    fn count_shown(&mut self) {
        let opponent = 1 - self.player;
        let mut piles = Vec::new();
        for (controller, side) in (0..).zip(&self.sides.0) {
            for pile in &side.piles {
                let [mut own, mut other] = [Vec::new(), pile.foreign.clone()];
                pile.places.iter().for_each(|slot| if slot.foreign { &mut other } else { &mut own }.push(slot.code));
                own.extend(&pile.unplaced);
                pile.foreign.iter().for_each(|&code| _ = remove_one(&mut own, code));
                piles.extend(if controller == opponent { own } else { other });
            }
        }
        let field = self.sides.0.iter().flat_map(|side| side.zones.iter().flatten().flatten());
        let held = field.flat_map(|held| std::iter::once(held).chain(&held.materials));
        let held = held.filter(|held| held.owner == Some(opponent)).map(|held| held.code);
        let mut codes: Vec<u32> = piles.into_iter().chain(held).filter(|&code| code != 0).collect();
        codes.sort_unstable();
        for run in codes.chunk_by(|a, b| a == b) {
            if let Some(&code) = run.first() {
                let count = self.shown.entry(code).or_insert(0);
                *count = (*count).max(run.len() as u32);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIRMED: u32 = 14558127;
    const OTHER: u32 = 97268402;

    fn place(controller: u8, location: u8, sequence: u32, position: u32) -> Vec<u8> {
        [&[controller, location][..], &sequence.to_ne_bytes(), &position.to_ne_bytes()].concat()
    }

    fn framed(messages: &[Vec<u8>]) -> Vec<u8> {
        messages
            .iter()
            .flat_map(|message| [&(message.len() as u32).to_ne_bytes()[..], message].concat())
            .collect()
    }

    fn confirm(sequence: u32) -> Vec<u8> {
        [
            &[MSG_CONFIRM_CARDS, 0][..],
            &1u32.to_ne_bytes(),
            &CONFIRMED.to_ne_bytes(),
            &[1, LOCATION_HAND],
            &sequence.to_ne_bytes(),
        ]
        .concat()
    }

    fn shuffle() -> Vec<u8> {
        [
            &[MSG_SHUFFLE_HAND, 1][..],
            &3u32.to_ne_bytes(),
            &[CONFIRMED, OTHER, OTHER].map(u32::to_ne_bytes).concat(),
        ]
        .concat()
    }

    fn moved(code: u32, from: Vec<u8>, to: Vec<u8>) -> Vec<u8> {
        [&[MSG_MOVE][..], &code.to_ne_bytes(), &from, &to, &0u32.to_ne_bytes()].concat()
    }

    // Player 1 sets the hand card at `sequence` in its Spell & Trap Zone 2.
    fn set(sequence: u32, code: u32) -> Vec<u8> {
        moved(code, place(1, LOCATION_HAND, sequence, POS_FACEDOWN), place(1, LOCATION_SZONE, 2, POS_FACEDOWN))
    }

    // What player 0 sees: player 1's hand of `hand` cards and, if `set`, a face-down card in its Spell & Trap Zone 2.
    fn observation(hand: u8, set: bool) -> Observation {
        let card = |location, sequence, position: u32| SeenCard {
            controller: 1,
            location,
            sequence,
            position: position as u8,
            code: 0,
            attack: 0,
            defense: 0,
        };
        let hand = (0..hand).map(|sequence| card(LOCATION_HAND, sequence, POS_FACEDOWN));
        Observation {
            cards: hand.chain(set.then(|| card(LOCATION_SZONE, 2, POS_FACEDOWN))).collect(),
            ..Observation::default()
        }
    }

    // Player 0's tracker after player 1's raw messages, each step masked for player 0 and followed by what it sees.
    fn tracked(steps: &[(Vec<Vec<u8>>, Observation)]) -> Tracker {
        let mut tracker = Tracker::default();
        tracker.sync(&observation(3, false));
        for (messages, observation) in steps {
            let mut masked = Vec::new();
            mask(&framed(messages), 0, &mut masked);
            tracker.apply(&masked, observation);
        }
        tracker
    }

    #[test]
    fn should_keep_a_confirmed_card_in_a_shuffled_hand_until_an_unseen_card_leaves() {
        let shuffled = tracked(&[(vec![confirm(1), shuffle()], observation(3, false))]);
        assert_eq!(shuffled.unplaced(1, LOCATION_HAND), [CONFIRMED]);
        let set = tracked(&[
            (vec![confirm(1), shuffle()], observation(3, false)),
            (vec![set(0, OTHER)], observation(2, true)),
        ]);
        let zone = observation(2, true).cards.last().map_or(0, |card| set.code_at(card));
        assert_eq!((set.unplaced(1, LOCATION_HAND), zone), (&[][..], 0));
        assert_eq!(set.shown().collect::<Vec<_>>(), [(CONFIRMED, 1)]);
    }

    #[test]
    fn should_follow_a_confirmed_card_set_from_an_unshuffled_hand() {
        let set = tracked(&[
            (vec![confirm(1)], observation(3, false)),
            (vec![set(1, CONFIRMED)], observation(2, true)),
        ]);
        let zone = observation(2, true).cards.last().map_or(0, |card| set.code_at(card));
        assert_eq!((zone, set.unplaced(1, LOCATION_HAND)), (CONFIRMED, &[][..]));
    }

    #[test]
    fn should_not_count_its_own_card_added_to_the_opponents_hand_as_the_opponents() {
        let added = moved(CONFIRMED, place(0, LOCATION_DECK, 0, POS_FACEDOWN), place(1, LOCATION_HAND, 3, POS_FACEDOWN));
        let shuffled = tracked(&[(vec![added, confirm(3), shuffle()], observation(4, false))]);
        assert_eq!(shuffled.shown().count(), 0);
    }
}
