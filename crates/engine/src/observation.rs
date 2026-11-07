// What a player may know of a duel, and duels resampled to agree with it. The visibility rules mirror what EDOPro's
// server sends each player: card::get_infos's QUERY_IS_PUBLIC to both, and to the controller its cards outside the Deck.
// Like the core, they keep no memory: a card confirmed earlier is known only while something keeps it public.
use std::sync::{Arc, Mutex};

use carddata::{Card as CardData, CardDatabase, Deck};
use serde::{Deserialize, Serialize};

use crate::action::{Action, BattleCommand, IdleCommand, Prompt};
use crate::card::{Card, CardId, CardSet, CardState, MAX_CARDS};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::Event;
use crate::field::{Cards8, Cards16};
use crate::processor::Unit;
use crate::rng::Xoshiro256StarStar;
use crate::script::lock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub player: u8,
    pub turn: i16,
    pub turn_player: u8,
    pub phase: u16,
    pub lp: [i32; 2],
    // Every card in a location, in the order of [Duel::located_cards].
    pub cards: Vec<SeenCard>,
    pub chain: Vec<SeenLink>,
    // The player a prompt waits for; the prompt and its choices are shown only to that player.
    pub acting: Option<u8>,
    pub prompt: Option<Prompt>,
    pub choices: Vec<Choice>,
    pub result: Option<(u8, u8)>,
}

// A card where it is, as the core's loc_info places it, and its code (0 if hidden from the player).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenCard {
    pub controller: u8,
    pub location: u8,
    pub sequence: u8,
    // The position, or for an Xyz material its index among the materials.
    pub position: u8,
    pub code: u32,
    // The current ATK and DEF of a face-up monster on the field, else 0.
    pub attack: i32,
    pub defense: i32,
}

// A chain link as MSG_CHAINING showed it, and the cards it targets where they are now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenLink {
    pub code: u32,
    pub controller: u8,
    pub location: u16,
    pub sequence: u32,
    pub description: u64,
    pub targets: Vec<SeenCard>,
}

// A legal action with the card it picks (its own Deck shown to the player picking) and the effect or option description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub action: Action,
    pub card: Option<SeenCard>,
    pub description: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DeterminizeError {
    #[error("player {0} is not in the duel")]
    Player(u8),
    #[error("the pool has no card left for a hidden card in location {0:#x}")]
    Pool(u8),
    #[error("card {0} is not in the database")]
    UnknownCard(u32),
    #[error("no sample in {RERUN_ATTEMPTS} reran the waiting function to what the player sees")]
    Rerun,
}

const RERUN_ATTEMPTS: usize = 16;

// What a hidden card may be resampled to: a card of the Extra Deck or not, of the kind its zone holds.
#[derive(Debug, Clone, Copy)]
enum Kind {
    Any,
    Monster,
    SpellTrap,
    FieldSpell,
}

impl Kind {
    fn fits(self, data: &CardData) -> bool {
        let card_type = data.card_type;
        match self {
            Self::Any => true,
            Self::Monster => card_type & TYPE_MONSTER != 0,
            Self::SpellTrap => card_type & (TYPE_SPELL | TYPE_TRAP) != 0 && card_type & TYPE_FIELD == 0,
            Self::FieldSpell => card_type & TYPE_FIELD != 0,
        }
    }
}

// Hidden cards taking the places of others (each card of `assigned` the place of the card of `cards` at its index), then
// hidden cards taking other codes.
#[derive(Debug)]
struct Changes {
    cards: Vec<CardId>,
    assigned: Vec<CardId>,
    recodes: Vec<(CardId, &'static CardData)>,
}

fn take(codes: &mut Vec<u32>, code: u32) -> bool {
    codes
        .iter()
        .position(|&other| other == code)
        .map(|at| codes.swap_remove(at))
        .is_some()
}

fn shuffle<T>(items: &mut [T], rng: &mut Xoshiro256StarStar) {
    for index in (1..items.len()).rev() {
        items.swap(index, rng.get_next_integer(0, index as i32) as usize);
    }
}

fn same_place(a: &CardState, b: &CardState) -> bool {
    (a.controler, a.location, a.sequence, a.position) == (b.controler, b.location, b.sequence, b.position)
}

impl Duel {
    pub fn observe(&self, player: u8) -> Observation {
        self.unchanged(|_| self.observe_now(player))
    }

    // Runs `read`, then puts back the effect ids and statuses and the field id counter its effect lookups changed, here
    // and in a waiting function's snapshot: a condition becoming available takes a new id, which orders later lookups.
    fn unchanged<T>(&self, read: impl FnOnce(Option<&Duel>) -> T) -> T {
        let saved = |duel: &Duel| (duel.infos.field_id.get(), duel.effects.cells());
        let snapshot = self.core.replay.snapshot.as_ref().map(lock);
        let snapshot = snapshot.as_deref();
        let (own, waiting) = (saved(self), snapshot.map(saved));
        let result = read(snapshot);
        for (duel, (field_id, cells)) in std::iter::once((self, own)).chain(snapshot.zip(waiting)) {
            duel.infos.field_id.set(field_id);
            duel.effects.restore_cells(&cells);
        }
        result
    }

    fn observe_now(&self, player: u8) -> Observation {
        let seen = |card| self.seen(card, player, false);
        let chain = self.core.current_chain.iter().map(|chain| {
            let effect = chain.triggering_effect;
            SeenLink {
                code: self.cards[self.get_handler(effect)].data.code,
                controller: chain.triggering_controler,
                location: chain.triggering_location,
                sequence: chain.triggering_sequence,
                description: self.effects[effect].description,
                targets: chain.target_cards.into_iter().flatten().map(seen).collect(),
            }
        });
        let prompt = self.prompt();
        let own = prompt.filter(|prompt| prompt.player() == player);
        Observation {
            player,
            turn: self.infos.turn_id,
            turn_player: self.infos.turn_player,
            phase: self.infos.phase,
            lp: [self.player[0].lp, self.player[1].lp],
            cards: self.located_cards().into_iter().map(seen).collect(),
            chain: chain.collect(),
            acting: prompt.map(Prompt::player),
            prompt: own,
            choices: own.map(|prompt| self.choices(prompt, player)).unwrap_or_default(),
            result: self.result,
        }
    }

    // Every card in a location: per player its Deck, hand, Monster Zones (an Xyz Monster followed by its materials),
    // Spell & Trap Zones, GY, banishment and Extra Deck, each in sequence order.
    pub(crate) fn located_cards(&self) -> Vec<CardId> {
        let mut cards = Vec::with_capacity(MAX_CARDS);
        for p in 0..2 {
            let player = &self.player[p];
            cards.extend(player.list_main.iter().chain(&player.list_hand));
            for &card in player.list_mzone.iter().chain(&player.list_szone).flatten() {
                cards.push(card);
                let mut materials: Cards8 = self.xyz_materials(card).iter().collect();
                materials.sort_by_key(|&material| self.cards[material].current.sequence);
                cards.extend(materials);
            }
            cards.extend(player.list_grave.iter().chain(&player.list_remove).chain(&player.list_extra));
        }
        cards
    }

    // card::get_infos's QUERY_IS_PUBLIC, or the player controls the card outside the Deck. An Xyz material shows with
    // its monster, as QUERY_OVERLAY_CARD lists the materials' codes.
    fn is_visible(&self, card: CardId, player: u8) -> bool {
        let target = &self.cards[card];
        if let Some(xyz) = target.overlay_target {
            return self.is_visible(xyz, player);
        }
        let current = &target.current;
        target.is_position(POS_FACEUP)
            || self.is_related_to_chains(card)
            || (current.controler == player && current.location != LOCATION_DECK)
            || (current.location & (LOCATION_HAND | LOCATION_ONFIELD) != 0 && self.is_affected_by_effect(card, EFFECT_PUBLIC).is_some())
    }

    fn is_related_to_chains(&self, card: CardId) -> bool {
        self.cards[card].is_status(STATUS_CHAINING)
            && self.core.current_chain.iter().any(|chain| {
                let effect = chain.triggering_effect;
                self.get_handler(effect) == card && self.is_has_relation_effect(card, effect)
            })
    }

    // The card as `player` sees it; a prompt shows the player picking all its own cards, its Deck's included.
    fn seen(&self, card: CardId, player: u8, picking: bool) -> SeenCard {
        let place = self.cards.get_info_location(card);
        let visible = self.is_visible(card, player) || (picking && place.controler == player);
        let monster = visible && place.location == LOCATION_MZONE && self.cards[card].is_position(POS_FACEUP);
        SeenCard {
            controller: place.controler,
            location: place.location,
            sequence: place.sequence as u8,
            position: place.position as u8,
            code: if visible { self.cards[card].data.code } else { 0 },
            attack: if monster { self.get_attack(card) } else { 0 },
            defense: if monster { self.get_defense(card) } else { 0 },
        }
    }

    fn choices(&self, prompt: Prompt, player: u8) -> Vec<Choice> {
        let core = &self.core;
        let pick = |cards: &[CardId], index: u16| cards.get(usize::from(index)).map(|&card| self.seen(card, player, true));
        let activation = |index: u16| {
            core.select_chains.get(usize::from(index)).map_or((None, 0), |chain| {
                let effect = chain.triggering_effect;
                (pick(&[self.get_handler(effect)], 0), self.effects[effect].description)
            })
        };
        let question = match core.units.last() {
            Some(Unit::SelectYesNo(arg)) => (None, arg.description),
            Some(Unit::SelectEffectYesNo(arg)) => (pick(&[arg.card], 0), arg.description),
            _ => (None, 0),
        };
        let selectable: Cards16 = core.select_cards.iter().chain(&core.unselect_cards).copied().collect();
        let choice = |action: Action| {
            let (card, description) = match action {
                Action::IdleCommand(IdleCommand::Summon(index)) => (pick(&core.summonable_cards, index), 0),
                Action::IdleCommand(IdleCommand::SpecialSummon(index)) => (pick(&core.spsummonable_cards, index), 0),
                Action::IdleCommand(IdleCommand::Reposition(index)) => (pick(&core.repositionable_cards, index), 0),
                Action::IdleCommand(IdleCommand::MonsterSet(index)) => (pick(&core.msetable_cards, index), 0),
                Action::IdleCommand(IdleCommand::SpellSet(index)) => (pick(&core.ssetable_cards, index), 0),
                Action::IdleCommand(IdleCommand::Activate(index))
                | Action::BattleCommand(BattleCommand::Activate(index))
                | Action::Chain(Some(index)) => activation(index),
                Action::BattleCommand(BattleCommand::Attack(index)) => (pick(&core.attackable_cards, index), 0),
                Action::Card(index) if matches!(prompt, Prompt::UnselectCard { .. }) => (pick(&selectable, index), 0),
                Action::Card(index) => (pick(&core.select_cards, index), 0),
                Action::Option(index) => (None, core.select_options.get(usize::from(index)).copied().unwrap_or(0)),
                Action::YesNo(_) => question,
                _ => (None, 0),
            };
            Choice { action, card, description }
        };
        self.legal_actions().into_iter().map(choice).collect()
    }

    // A duel agreeing with what `player` may know of this one ([Duel::observe]), its hidden cards resampled: the
    // opponent's hand, Deck, face-down cards and face-down Extra Deck from `pool` (its Main and Extra Deck as known or
    // assumed) less the opponent's cards the player knows, the player's own Deck reshuffled, and the duel's RNG reseeded.
    // Hidden cards that something in the duel refers to as the cards they are keep their place and code
    // ([Duel::referenced_cards]), and a face-down card keeps whether it is an Extra Deck card.
    // While a function waits midway for an answer, its snapshot is resampled instead (the cards it has moved since
    // keep theirs) and the function reruns to its prompt, as an answer would rerun it; a sample whose rerun shows the
    // player anything else, or stops at another prompt or one without actions (the opponent's earlier answers can lead
    // elsewhere in it), is drawn again, up to [RERUN_ATTEMPTS] times. Its RNG stays, as the rerun replays it.
    pub fn determinize(&self, player: u8, rng: &mut Xoshiro256StarStar, pool: &Deck) -> Result<Duel, DeterminizeError> {
        if player > 1 {
            return Err(DeterminizeError::Player(player));
        }
        let mut duel = self.clone();
        let Some(database) = self.database.filter(|_| self.result.is_none()) else {
            return Ok(duel);
        };
        if self.core.replay.snapshot.is_none() {
            let changes = self.unchanged(|_| self.sample(player, rng, pool, database, None))?;
            let trace = std::mem::take(&mut duel.trace);
            duel.change(&changes);
            duel.trace = trace;
            duel.random = Xoshiro256StarStar::new([rng.next_u64(), rng.next_u64(), rng.next_u64(), rng.next_u64() | 1]);
            return Ok(duel);
        }
        let (observation, prompt) = (self.observe(player), self.prompt());
        for _ in 0..RERUN_ATTEMPTS {
            let changes = self.unchanged(|running| self.sample(player, rng, pool, database, running))?;
            if let Some(snapshot) = &mut duel.core.replay.snapshot {
                let mut resampled = lock(snapshot).clone();
                resampled.change(&changes);
                *snapshot = Arc::new(Mutex::new(resampled));
            }
            duel.rerun_to_prompt();
            if duel.prompt() == prompt && duel.observe(player) == observation && !duel.legal_actions().is_empty() {
                return Ok(duel);
            }
            duel = self.clone();
        }
        Err(DeterminizeError::Rerun)
    }

    // What a function waiting midway for an answer does once answered, without the answer: it reruns from its snapshot
    // with the answers so far, up to the prompt that waits, whose picks so far are kept.
    fn rerun_to_prompt(&mut self) {
        self.core.units.pop();
        let picks = std::mem::take(&mut self.returns);
        self.returns.data = self.core.replay.log.pop().unwrap_or_default();
        self.core.replay.messages = self.trace.count;
        let buffer = self.trace.buffer.take();
        self.run();
        self.trace.buffer = buffer;
        self.returns = picks;
    }

    // The changes that resample what `player` cannot see (see [Duel::determinize]): the codes the opponent's hidden
    // cards are to show, drawn for the zones that restrict them first, then the hand and Deck, and a new order of the
    // player's own Deck.
    fn sample(
        &self,
        player: u8,
        rng: &mut Xoshiro256StarStar,
        pool: &Deck,
        database: &'static CardDatabase,
        running: Option<&Duel>,
    ) -> Result<Changes, DeterminizeError> {
        let opponent = 1 - player;
        let pinned = self.referenced_cards();
        let resampled = |card: CardId| {
            let target = &self.cards[card];
            let unmoved = running
                .is_none_or(|snapshot| same_place(&snapshot.cards[card].current, &target.current) && !snapshot.is_visible(card, player));
            !pinned.contains(card)
                && target.current.controler == target.owner
                && target.overlay_target.is_none()
                && unmoved
                && !self.is_visible(card, player)
                && self
                    .card_effects(card)
                    .iter()
                    .all(|&effect| self.effects[effect].is_flag(EFFECT_FLAG_INITIAL))
        };
        let (mut main, mut extra) = (pool.main.clone(), pool.extra.clone());
        let (mut slots, mut piles, mut own) = (Vec::new(), Vec::new(), Vec::new());
        for card in self.located_cards() {
            let target = &self.cards[card];
            let (owner, current) = (target.owner, target.current);
            match (owner == opponent, resampled(card)) {
                (true, true) if current.location & (LOCATION_HAND | LOCATION_DECK) != 0 => piles.push(card),
                (true, true) => slots.push((card, self.resampled_kind(card))),
                (true, false) => {
                    if !take(&mut main, target.data.code) {
                        take(&mut extra, target.data.code);
                    }
                }
                (false, true) => own.push(card),
                (false, false) => {}
            }
        }
        let mut targets = Vec::new();
        for &(card, (from_extra, kind)) in &slots {
            let list = if from_extra { &mut extra } else { &mut main };
            let fits = |code: &u32| database.get(*code).is_some_and(|data| kind.fits(data));
            let count = list.iter().filter(|code| fits(code)).count();
            if count == 0 {
                return Err(DeterminizeError::Pool(self.cards[card].current.location));
            }
            let pick = rng.get_next_integer(0, count as i32 - 1) as usize;
            let at = list
                .iter()
                .enumerate()
                .filter(|(_, code)| fits(code))
                .nth(pick)
                .map_or(0, |(at, _)| at);
            targets.push((card, list.swap_remove(at)));
        }
        shuffle(&mut main, rng);
        if main.len() < piles.len() {
            return Err(DeterminizeError::Pool(LOCATION_HAND | LOCATION_DECK));
        }
        targets.extend(piles.into_iter().zip(main));
        let mut own_codes: Vec<u32> = own.iter().map(|&card| self.cards[card].data.code).collect();
        shuffle(&mut own_codes, rng);
        targets.extend(own.into_iter().zip(own_codes));
        let (cards, codes): (Vec<CardId>, Vec<u32>) = targets.into_iter().unzip();
        let (assigned, recodes) = self.arrange(&cards, &codes);
        let recodes = recodes.into_iter().map(|(card, code)| {
            let data = database.get(code).ok_or(DeterminizeError::UnknownCard(code))?;
            Ok((card, data))
        });
        Ok(Changes {
            recodes: recodes.collect::<Result<_, _>>()?,
            cards,
            assigned,
        })
    }

    // A hidden card outside the hand and Deck: whether it is an Extra Deck card, and the kind its zone holds.
    fn resampled_kind(&self, card: CardId) -> (bool, Kind) {
        let target = &self.cards[card];
        let extra = self.is_extra_deck_monster(card);
        match target.current.location {
            LOCATION_EXTRA => (true, Kind::Any),
            LOCATION_MZONE => (extra, Kind::Monster),
            LOCATION_SZONE if target.current.sequence == 5 => (false, Kind::FieldSpell),
            LOCATION_SZONE if target.data.card_type & (TYPE_SPELL | TYPE_TRAP) != 0 => (false, Kind::SpellTrap),
            LOCATION_SZONE => (extra, Kind::Monster),
            _ => (extra, Kind::Any),
        }
    }

    // The cards to put in the places of `cards` so that they show the `targets` codes, and the cards to recode: a card
    // of the right code trades places where one is left, which keeps its effects as they are.
    fn arrange(&self, cards: &[CardId], targets: &[u32]) -> (Vec<CardId>, Vec<(CardId, u32)>) {
        let mut free: Vec<(u32, Option<CardId>)> = cards.iter().map(|&card| (self.cards[card].data.code, Some(card))).collect();
        free.sort_unstable();
        let mut assigned: Vec<Option<CardId>> = targets
            .iter()
            .map(|&target| {
                let start = free.partition_point(|&(other, _)| other < target);
                let mut same = free.get_mut(start..)?.iter_mut().take_while(|(other, _)| *other == target);
                same.find_map(|(_, card)| card.take())
            })
            .collect();
        let mut leftovers = free.into_iter().filter_map(|(_, card)| card);
        let mut recodes = Vec::new();
        for (slot, &target) in assigned.iter_mut().zip(targets) {
            if slot.is_none() {
                *slot = leftovers.next();
                recodes.extend(slot.map(|card| (card, target)));
            }
        }
        (assigned.into_iter().flatten().collect(), recodes)
    }

    fn change(&mut self, changes: &Changes) {
        self.rearrange(&changes.cards, &changes.assigned);
        for &(card, data) in &changes.recodes {
            self.recode(card, data);
        }
    }

    // Each card of `assigned` takes the place of the card of `cards` at its index: its location, position, status and
    // the rest of what the place holds, with its field effects leaving and joining the field when the location changes.
    // Pending events then name the card in the place of the one they named, as a drawn card stands for whatever was drawn.
    fn rearrange(&mut self, cards: &[CardId], assigned: &[CardId]) {
        let places: Vec<Card> = cards.iter().map(|&card| self.cards[card]).collect();
        let moving: Vec<bool> = places
            .iter()
            .zip(assigned)
            .map(|(place, &card)| self.cards[card].current.location != place.current.location)
            .collect();
        for (&card, _) in assigned.iter().zip(&moving).filter(|(_, moving)| **moving) {
            self.cancel_field_effect(card);
        }
        for (place, &card) in places.iter().zip(assigned) {
            let data = self.cards[card].data;
            self.cards[card] = Card { data, ..*place };
            let CardState {
                controler,
                location,
                sequence,
                ..
            } = place.current;
            if location & LOCATION_ONFIELD != 0 {
                if let Some(zone) = self.player[controler].zone_mut(location, sequence) {
                    *zone = Some(card);
                }
            } else if let Some(slot) = self.list_mut(controler, location).and_then(|list| list.get_mut(sequence as usize)) {
                *slot = card;
            }
        }
        for (&card, _) in assigned.iter().zip(&moving).filter(|(_, moving)| **moving) {
            self.apply_field_effect(card);
        }
        self.remap_events(cards, assigned);
    }

    fn remap_events(&mut self, cards: &[CardId], assigned: &[CardId]) {
        let group: CardSet = cards.iter().copied().collect();
        let map = |card: CardId| match cards.iter().position(|&other| other == card) {
            Some(at) => assigned.get(at).copied().unwrap_or(card),
            None => card,
        };
        let remap = |event: &mut Event| {
            if let Some(set) = event.event_cards.as_mut().filter(|set| !set.intersection(group).is_empty()) {
                *set = set.iter().map(map).collect();
            }
            if let Some(card) = event.trigger_card.as_mut().filter(|card| group.contains(**card)) {
                *card = map(*card);
            }
        };
        let core = &mut self.core;
        let events: [&mut [Event]; 8] = [
            &mut core.point_event,
            &mut core.instant_event,
            &mut core.queue_event,
            &mut core.delayed_activate_event,
            &mut core.full_event,
            &mut core.single_event,
            &mut core.solving_event,
            &mut core.sub_solving_event,
        ];
        events.into_iter().flatten().for_each(remap);
        let quick = core.delayed_quick.iter_mut().chain(&mut core.delayed_quick_tmp);
        quick.for_each(|(_, event)| remap(event));
        let chains = [
            &mut core.select_chains,
            &mut core.current_chain,
            &mut core.ignition_priority_chains,
            &mut core.continuous_chain,
            &mut core.solving_continuous,
            &mut core.sub_solving_continuous,
            &mut core.delayed_continuous_tp,
            &mut core.delayed_continuous_ntp,
            &mut core.continuous_tp,
            &mut core.continuous_ntp,
            &mut core.new_fchain,
            &mut core.new_fchain_s,
            &mut core.new_ochain,
            &mut core.new_ochain_s,
            &mut core.new_fchain_b,
            &mut core.new_ochain_b,
            &mut core.new_ochain_h,
            &mut core.new_chains,
        ];
        let chains = chains.into_iter().flatten();
        chains
            .chain(core.quick_f_chain.iter_mut().map(|(_, chain)| chain))
            .for_each(|chain| remap(&mut chain.evt));
    }

    // card::replace_effect for Card.Recreate, without its traces: the card drops the effects its script registered and
    // its new code's script registers them.
    fn recode(&mut self, card: CardId, data: &'static CardData) {
        for effect in self.card_effects(card) {
            if self.effects[effect].is_flag(EFFECT_FLAG_INITIAL) {
                self.card_remove_effect(card, effect);
            }
        }
        self.cards[card].data = data.into();
        self.register_card(card);
    }

    // The cards something refers to as the cards they are, depending on their code or effects: the prompt's cards, the
    // chains' cards and targets, relations, materials, counters, attack announcements and the groups scripts keep.
    fn referenced_cards(&self) -> CardSet {
        let core = &self.core;
        let prompt = self.prompt();
        let offered = matches!(
            prompt,
            Some(Prompt::IdleCommand { .. } | Prompt::BattleCommand { .. } | Prompt::Chain { .. })
        );
        let chains = [
            &core.current_chain,
            &core.ignition_priority_chains,
            &core.continuous_chain,
            &core.solving_continuous,
            &core.sub_solving_continuous,
            &core.delayed_continuous_tp,
            &core.delayed_continuous_ntp,
            &core.new_fchain,
            &core.new_fchain_s,
            &core.new_ochain,
            &core.new_ochain_s,
            &core.new_fchain_b,
            &core.new_ochain_b,
            &core.new_ochain_h,
            &core.new_chains,
        ];
        let offered_chains = offered.then_some(&core.select_chains);
        let chains = chains.into_iter().chain(offered_chains).flatten();
        let chains = chains.chain(core.quick_f_chain.iter().map(|(_, chain)| chain));
        let mut cards = chains.fold(CardSet::default(), |cards, chain| {
            let handler = CardSet::from(self.get_handler(chain.triggering_effect));
            cards.union(handler).union(chain.target_cards.unwrap_or_default())
        });
        let lists: [&[CardId]; 5] = match prompt {
            Some(Prompt::IdleCommand { .. }) => [
                &core.summonable_cards,
                &core.spsummonable_cards,
                &core.repositionable_cards,
                &core.msetable_cards,
                &core.ssetable_cards,
            ],
            Some(Prompt::BattleCommand { .. }) => [&core.attackable_cards, &[], &[], &[], &[]],
            Some(Prompt::Card { .. } | Prompt::Tribute { .. } | Prompt::Sum { .. } | Prompt::UnselectCard { .. } | Prompt::Sort { .. }) => {
                cards = cards.union(core.must_select_cards.get());
                [&core.select_cards, &core.unselect_cards, &[], &[], &[]]
            }
            _ => [&[]; 5],
        };
        let others = [core.attacker, core.attack_target, core.chain_attack_target, core.summoning_card];
        let effect_yes_no = match core.units.last() {
            Some(Unit::SelectEffectYesNo(arg)) => Some(arg.card),
            _ => None,
        };
        let singles = lists
            .into_iter()
            .flatten()
            .copied()
            .chain(others.into_iter().flatten())
            .chain(effect_yes_no);
        let counted = self.counters.iter().map(|&(card, ..)| card);
        let singles = singles.chain(counted).chain(self.announced_cards.iter().map(|&(card, ..)| card));
        let pairs = self.card_relations.iter().map(|&(card, other, _)| [card, other]);
        let pairs = pairs.chain(self.card_targets.iter().map(|&(card, target)| [card, target]));
        let related = self.relations.iter().map(|relation| relation.card);
        for card in singles.chain(pairs.flatten()).chain(related) {
            cards.insert(card);
        }
        for (card, materials) in &self.material_cards {
            cards.insert(*card);
            cards = cards.union(*materials);
        }
        self.script_groups.iter().fold(cards, |cards, (_, group)| cards.union(*group))
    }
}
