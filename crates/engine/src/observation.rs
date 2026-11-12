// What a player may know of a duel, as EDOPro's server shows it (no memory of earlier confirms), and duels resampled to agree.
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use carddata::{Card as CardData, CardDatabase, Deck};
use serde::{Deserialize, Serialize};

use crate::action::{Action, BattleCommand, IdleCommand, Prompt};
use crate::card::{Card, CardId, CardSet, CardState, MAX_CARDS};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{CellChunk, Event};
use crate::field::Cards8;
use crate::list::List;
use crate::processor::Unit;
use crate::rng::Xoshiro256StarStar;
use crate::script::lock;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

pub const MAX_DECLARED_CODES: usize = 64;

thread_local! {
    // [Duel::unchanged]'s saved effect ids and statuses, kept so that observing allocates nothing.
    static SAVED: Cell<Vec<CellChunk>> = const { Cell::new(Vec::new()) };
}

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

// Hidden cards taking the places of others (`assigned[i]` the place of `cards[i]`), then hidden cards taking other codes.
#[derive(Debug)]
struct Changes {
    cards: Vec<CardId>,
    assigned: Vec<CardId>,
    recodes: Vec<(CardId, &'static CardData)>,
}

// A duel's hidden cards for a player, found once for any number of samples ([Duel::determinizer]).
pub struct Determinizer<'a> {
    duel: &'a Duel,
    player: u8,
    // None if the duel ended or has no database to draw codes from: samples are plain copies.
    hidden: Option<Hidden>,
    // What a waiting function's rerun must show the player again.
    shown: Option<(Observation, Option<Prompt>)>,
}

// Pool codes with their data, looked up once.
type Pool = Vec<(u32, Option<&'static CardData>)>;

// The opponent's hidden cards outside the hand and Deck (with the kind each takes), in the hand and Deck, the player's own hidden
// cards, and the pool codes no card the player sees holds.
struct Hidden {
    database: &'static CardDatabase,
    slots: Vec<(CardId, (bool, Kind))>,
    piles: Vec<CardId>,
    own: Vec<CardId>,
    main: Pool,
    extra: Pool,
}

impl Determinizer<'_> {
    pub fn determinize(&self, rng: &mut Xoshiro256StarStar) -> Result<Duel, DeterminizeError> {
        let mut duel = self.duel.clone();
        self.resample(rng, &mut duel)?;
        Ok(duel)
    }

    // [Determinizer::determinize] into `duel`, reusing its buffers.
    pub fn determinize_into(&self, rng: &mut Xoshiro256StarStar, duel: &mut Duel) -> Result<(), DeterminizeError> {
        duel.clone_from(self.duel);
        self.resample(rng, duel)
    }

    // `duel` is a copy of the determinized duel.
    fn resample(&self, rng: &mut Xoshiro256StarStar, duel: &mut Duel) -> Result<(), DeterminizeError> {
        let Some(hidden) = &self.hidden else {
            return Ok(());
        };
        let Some((observation, prompt)) = &self.shown else {
            let changes = self.draw(hidden, rng)?;
            let trace = std::mem::take(&mut duel.trace);
            duel.change(&changes);
            duel.trace = trace;
            duel.random = Xoshiro256StarStar::new([rng.next_u64(), rng.next_u64(), rng.next_u64(), rng.next_u64() | 1]);
            return Ok(());
        };
        for _ in 0..RERUN_ATTEMPTS {
            let changes = self.draw(hidden, rng)?;
            if let Some(snapshot) = &mut duel.replay.snapshot {
                let mut resampled = lock(snapshot).clone();
                resampled.change(&changes);
                *snapshot = Arc::new(Mutex::new(resampled));
            }
            duel.rerun_to_prompt();
            if duel.prompt() == *prompt && duel.observe(self.player) == *observation && !duel.legal_actions().is_empty() {
                return Ok(());
            }
            duel.clone_from(self.duel);
        }
        Err(DeterminizeError::Rerun)
    }

    // Draws the opponent's hidden codes, restricted zones first, then hand and Deck, and a new order of the player's own Deck.
    fn draw(&self, hidden: &Hidden, rng: &mut Xoshiro256StarStar) -> Result<Changes, DeterminizeError> {
        let duel = self.duel;
        let (mut main, mut extra) = (hidden.main.clone(), hidden.extra.clone());
        let mut targets = Vec::with_capacity(hidden.slots.len() + hidden.piles.len() + hidden.own.len());
        for &(card, (from_extra, kind)) in &hidden.slots {
            let list = if from_extra { &mut extra } else { &mut main };
            let fits = |&(_, data): &(u32, Option<&CardData>)| data.is_some_and(|data| kind.fits(data));
            let count = list.iter().filter(|code| fits(code)).count();
            if count == 0 {
                return Err(DeterminizeError::Pool(duel.cards[card].current.location));
            }
            let pick = rng.get_next_integer(0, count as i32 - 1) as usize;
            let at = list
                .iter()
                .enumerate()
                .filter(|(_, code)| fits(code))
                .nth(pick)
                .map_or(0, |(at, _)| at);
            targets.push((card, list.swap_remove(at).0));
        }
        shuffle(&mut main, rng);
        if main.len() < hidden.piles.len() {
            return Err(DeterminizeError::Pool(LOCATION_HAND | LOCATION_DECK));
        }
        targets.extend(hidden.piles.iter().copied().zip(main.into_iter().map(|(code, _)| code)));
        let mut own_codes: Vec<u32> = hidden.own.iter().map(|&card| duel.cards[card].data.code).collect();
        shuffle(&mut own_codes, rng);
        targets.extend(hidden.own.iter().copied().zip(own_codes));
        let (cards, codes): (Vec<CardId>, Vec<u32>) = targets.into_iter().unzip();
        let (assigned, recodes) = duel.arrange(&cards, &codes);
        let recodes = recodes.into_iter().map(|(card, code)| {
            let data = hidden.database.get(code).ok_or(DeterminizeError::UnknownCard(code))?;
            Ok((card, data))
        });
        Ok(Changes {
            recodes: recodes.collect::<Result<_, _>>()?,
            cards,
            assigned,
        })
    }
}

fn take(codes: &mut Pool, code: u32) -> bool {
    codes
        .iter()
        .position(|&(other, _)| other == code)
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

// Receives an observation piece by piece in the order of [Observation]'s lists, each chain link (without) then its targets.
pub trait ObservationSink {
    fn card(&mut self, card: SeenCard);
    fn link(&mut self, link: &SeenLink);
    fn target(&mut self, card: SeenCard);
    fn choice(&mut self, choice: Choice);
}

impl ObservationSink for Observation {
    fn card(&mut self, card: SeenCard) {
        self.cards.push(card);
    }

    fn link(&mut self, link: &SeenLink) {
        self.chain.push(link.clone());
    }

    fn target(&mut self, card: SeenCard) {
        if let Some(link) = self.chain.last_mut() {
            link.targets.push(card);
        }
    }

    fn choice(&mut self, choice: Choice) {
        self.choices.push(choice);
    }
}

// Passes each item it is extended with to a function, so that legal actions stream without a list.
struct Each<F>(F);

impl<T, F: FnMut(T)> Extend<T> for Each<F> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
        items.into_iter().for_each(&mut self.0);
    }
}

impl Duel {
    pub fn observe(&self, player: u8) -> Observation {
        let mut lists = Observation {
            cards: Vec::with_capacity(MAX_CARDS),
            ..Observation::default()
        };
        let observation = self.observe_into(player, &mut lists);
        Observation {
            cards: lists.cards,
            chain: lists.chain,
            choices: lists.choices,
            ..observation
        }
    }

    // [Duel::observe] with the same lookups, its lists fed to `sink` instead; returns the rest, the lists left empty.
    pub fn observe_into(&self, player: u8, sink: &mut impl ObservationSink) -> Observation {
        self.unchanged(|| self.observe_now(player, sink))
    }

    // Runs `read`, then puts back the effect ids, statuses, field id counter and script state its lookups changed.
    fn unchanged<T>(&self, read: impl FnOnce() -> T) -> T {
        let mut saved = SAVED.take();
        self.effects.save_cells(&mut saved);
        let field_id = self.infos.field_id.get();
        let script_state = self.script_state.borrow().clone();
        let result = read();
        self.infos.field_id.set(field_id);
        *self.script_state.borrow_mut() = script_state;
        self.effects.restore_cells(&saved);
        SAVED.set(saved);
        result
    }

    fn observe_now(&self, player: u8, sink: &mut impl ObservationSink) -> Observation {
        let seen = |card| self.seen(card, player, false);
        self.located_cards().into_iter().for_each(|card| sink.card(seen(card)));
        for chain in &self.core.current_chain {
            let effect = chain.triggering_effect;
            sink.link(&SeenLink {
                code: self.cards[self.get_handler(effect)].data.code,
                controller: chain.triggering_controler,
                location: chain.triggering_location,
                sequence: chain.triggering_sequence,
                description: self.effects[effect].description,
                targets: Vec::new(),
            });
            chain.target_cards.into_iter().flatten().for_each(|card| sink.target(seen(card)));
        }
        let prompt = self.prompt();
        let own = prompt.filter(|prompt| prompt.player() == player);
        if let Some(prompt) = own {
            self.choices(prompt, player, sink);
        }
        Observation {
            player,
            turn: self.infos.turn_id,
            turn_player: self.infos.turn_player,
            phase: self.infos.phase,
            lp: [self.player[0].lp, self.player[1].lp],
            acting: prompt.map(Prompt::player),
            prompt: own,
            result: self.result,
            ..Observation::default()
        }
    }

    // Per player: Deck, hand, Monster Zones (each Xyz Monster then its materials), S/T Zones, GY, banished and Extra Deck.
    pub(crate) fn located_cards(&self) -> List<CardId, MAX_CARDS> {
        let mut cards = List::default();
        for p in 0..2 {
            let player = &self.player[p];
            cards.extend(player.list_main.iter().chain(&player.list_hand).copied());
            for &card in player.list_mzone.iter().chain(&player.list_szone).flatten() {
                cards.push(card);
                let mut materials: Cards8 = self.xyz_materials(card).iter().collect();
                materials.sort_by_key(|&material| self.cards[material].current.sequence);
                cards.extend(materials);
            }
            cards.extend(
                player
                    .list_grave
                    .iter()
                    .chain(&player.list_remove)
                    .chain(&player.list_extra)
                    .copied(),
            );
        }
        cards
    }

    // QUERY_IS_PUBLIC, or the player controls the card outside the Deck; a material shows with its monster (QUERY_OVERLAY_CARD).
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

    fn choices(&self, prompt: Prompt, player: u8, sink: &mut impl ObservationSink) {
        let choice = |action: Action| {
            let (card, description) = self.choice_card(prompt, action);
            let card = card.map(|card| self.seen(card, player, true));
            Choice { action, card, description }
        };
        self.extend_policy_actions(prompt, &mut Each(|action| sink.choice(choice(action))));
    }

    // The card an action picks and the effect or option description, as [Duel::observe]'s choices show them, and the card a
    // Place or Position prompt is for ([Duel::prompt_card]); without the lookups of an observation.
    pub fn action_card(&self, action: Action) -> (Option<CardId>, u64) {
        match (self.prompt(), action) {
            (Some(_), Action::Place(_) | Action::Position(_)) => (self.prompt_card(), 0),
            (Some(prompt), _) => self.choice_card(prompt, action),
            (None, _) => (None, 0),
        }
    }

    fn choice_card(&self, prompt: Prompt, action: Action) -> (Option<CardId>, u64) {
        let core = &self.core;
        let pick = |cards: &[CardId], index: u16| cards.get(usize::from(index)).copied();
        match action {
            Action::IdleCommand(IdleCommand::Activate(index))
            | Action::BattleCommand(BattleCommand::Activate(index))
            | Action::Chain(Some(index)) => core.select_chains.get(usize::from(index)).map_or((None, 0), |chain| {
                let effect = chain.triggering_effect;
                (Some(self.get_handler(effect)), self.effects[effect].description)
            }),
            Action::IdleCommand(command) => (self.idle_card(command), 0),
            Action::BattleCommand(BattleCommand::Attack(index)) => (pick(&core.attackable_cards, index), 0),
            Action::Card(index) if matches!(prompt, Prompt::UnselectCard { .. }) => (
                core.select_cards
                    .iter()
                    .chain(&core.unselect_cards)
                    .nth(usize::from(index))
                    .copied(),
                0,
            ),
            Action::Card(index) => (pick(&core.select_cards, index), 0),
            Action::Option(index) => (None, core.select_options.get(usize::from(index)).copied().unwrap_or(0)),
            Action::YesNo(_) => match core.units.last() {
                Some(Unit::SelectYesNo(arg)) => (None, arg.description),
                Some(Unit::SelectEffectYesNo(arg)) => (Some(arg.card), arg.description),
                _ => (None, 0),
            },
            _ => (None, 0),
        }
    }

    pub fn policy_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        self.policy_actions_into(&mut actions);
        actions
    }

    // [Duel::legal_actions_into] as a policy chooses among them, as [Duel::observe]'s choices list them.
    pub fn policy_actions_into(&self, actions: &mut Vec<Action>) {
        actions.clear();
        match self.prompt() {
            Some(prompt @ Prompt::AnnounceCard { .. }) => self.unchanged(|| self.extend_policy_actions(prompt, actions)),
            _ => self.extend_legal_actions(actions),
        }
    }

    // A card-name declaration offers only the codes its player knows, else the first declarable ones, up to [MAX_DECLARED_CODES].
    fn extend_policy_actions(&self, prompt: Prompt, actions: &mut impl Extend<Action>) {
        let Prompt::AnnounceCard { player } = prompt else {
            return self.extend_legal_actions(actions);
        };
        let mut known: Vec<u32> = self
            .located_cards()
            .into_iter()
            .map(|card| match self.cards[card].owner == player {
                true => self.cards[card].data.code,
                false => self.seen(card, player, false).code,
            })
            .collect();
        known.sort_unstable();
        let declarable = || {
            let codes = self.database.into_iter().flat_map(|database| database.cards());
            codes.map(|card| card.code).filter(|&code| self.is_declarable(code))
        };
        let mut known_declarable = declarable().filter(|code| known.binary_search(code).is_ok()).peekable();
        let fallback = known_declarable.peek().is_none().then(declarable).into_iter().flatten();
        actions.extend(known_declarable.chain(fallback).take(MAX_DECLARED_CODES).map(Action::Code));
    }

    // A duel agreeing with what `player` sees, its hidden cards resampled from `pool` (in a waiting function's snapshot, rerun).
    pub fn determinize(&self, player: u8, rng: &mut Xoshiro256StarStar, pool: &Deck) -> Result<Duel, DeterminizeError> {
        self.determinizer(player, pool)?.determinize(rng)
    }

    // [Duel::determinize] into `duel`, reusing its buffers.
    pub fn determinize_into(&self, player: u8, rng: &mut Xoshiro256StarStar, pool: &Deck, duel: &mut Duel) -> Result<(), DeterminizeError> {
        self.determinizer(player, pool)?.determinize_into(rng, duel)
    }

    // What [Duel::determinize] finds before it draws, for drawing many samples of this duel.
    pub fn determinizer(&self, player: u8, pool: &Deck) -> Result<Determinizer<'_>, DeterminizeError> {
        if player > 1 {
            return Err(DeterminizeError::Player(player));
        }
        let hidden = self.database.filter(|_| self.result.is_none()).map(|database| {
            let running = self.replay.snapshot.as_ref().map(lock);
            self.unchanged(|| match running.as_deref() {
                Some(running) => running.unchanged(|| self.hidden_cards(player, pool, database, Some(running))),
                None => self.hidden_cards(player, pool, database, None),
            })
        });
        let shown = (hidden.is_some() && self.replay.snapshot.is_some()).then(|| (self.observe(player), self.prompt()));
        Ok(Determinizer {
            duel: self,
            player,
            hidden,
            shown,
        })
    }

    // Reruns a waiting function from its snapshot up to its prompt, without a new answer, keeping the picks so far.
    fn rerun_to_prompt(&mut self) {
        self.core.units.pop();
        let picks = std::mem::take(&mut self.returns);
        self.returns.data = self.replay.log.pop().unwrap_or_default();
        self.replay.messages = self.trace.count.get();
        let buffer = self.trace.buffer.take();
        self.run();
        *self.trace.buffer.get_mut() = buffer;
        self.returns = picks;
    }

    // The opponent's hidden cards by where their codes come from and the player's own resampled cards, with the pool codes left.
    fn hidden_cards(&self, player: u8, pool: &Deck, database: &'static CardDatabase, running: Option<&Duel>) -> Hidden {
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
                && (self.field_effects.registry.of_card(card)).all(|effect| self.effects[effect].is_flag(EFFECT_FLAG_INITIAL))
        };
        let with_data = |codes: &[u32]| -> Pool { codes.iter().map(|&code| (code, database.get(code))).collect() };
        let located = self.located_cards();
        let mut hidden = Hidden {
            database,
            slots: Vec::with_capacity(located.len()),
            piles: Vec::with_capacity(located.len()),
            own: Vec::with_capacity(located.len()),
            main: with_data(&pool.main),
            extra: with_data(&pool.extra),
        };
        for card in located {
            let target = &self.cards[card];
            let (owner, current) = (target.owner, target.current);
            match (owner == opponent, resampled(card)) {
                (true, true) if current.location & (LOCATION_HAND | LOCATION_DECK) != 0 => hidden.piles.push(card),
                (true, true) => hidden.slots.push((card, self.resampled_kind(card))),
                (true, false) => {
                    if !take(&mut hidden.main, target.data.code) {
                        take(&mut hidden.extra, target.data.code);
                    }
                }
                (false, true) => hidden.own.push(card),
                (false, false) => {}
            }
        }
        hidden
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

    // The cards to place on `cards` to show the `targets` codes, preferring a card of the right code, and those to recode.
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

    // Two cards trade places and fields as [Duel::determinize] trades hidden cards, while everything else that refers to either
    // keeps its id; for checking that [Duel::state_hash] tells copies of a card apart.
    pub fn swap_cards(&mut self, card: CardId, other: CardId) {
        let trace = std::mem::take(&mut self.trace);
        self.rearrange(&[card, other], &[other, card]);
        self.trace = trace;
    }

    // `assigned[i]` takes over everything `cards[i]`'s place holds, field effects and pending events included.
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
        let continuous = [
            &mut core.solving_continuous,
            &mut core.sub_solving_continuous,
            &mut core.delayed_continuous_tp,
            &mut core.delayed_continuous_ntp,
            &mut core.continuous_tp,
            &mut core.continuous_ntp,
        ];
        continuous.into_iter().flatten().for_each(|continuous| remap(&mut continuous.event));
    }

    // card::replace_effect for Card.Recreate, without its messages.
    fn recode(&mut self, card: CardId, data: &'static CardData) {
        for effect in self.card_effects(card) {
            if self.effects[effect].is_flag(EFFECT_FLAG_INITIAL) {
                self.card_remove_effect(card, effect);
            }
        }
        self.cards[card].data = data.into();
        self.register_card(card);
    }

    // The cards something in the duel refers to as the cards they are, so they keep their place and code when resampled.
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
        let waiting = [
            &core.solving_continuous,
            &core.sub_solving_continuous,
            &core.delayed_continuous_tp,
            &core.delayed_continuous_ntp,
        ];
        let waiting = waiting.into_iter().flatten().map(|continuous| self.get_handler(continuous.effect));
        let mut cards = chains.fold(waiting.collect(), |cards: CardSet, chain| {
            let handler = CardSet::from(self.get_handler(chain.triggering_effect));
            cards.union(handler).union(chain.target_cards.unwrap_or_default())
        });
        let lists: [&[CardId]; 5] = match prompt {
            Some(Prompt::IdleCommand { .. }) => self.idle_lists(),
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
        let singles = singles
            .chain(counted)
            .chain(self.announced_cards.iter().map(|&(card, ..)| card))
            .chain(self.attacked_cards.iter().map(|&(card, _)| card))
            .chain(self.indestructable_effects.iter().map(|&(card, ..)| card))
            .chain(self.battled_cards.iter().map(|&(card, _)| card));
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
