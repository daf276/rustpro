use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;
use std::path::Path;

use carddata::{CardDatabase, CardsError, Deck, Pool, TYPE_NORMAL, TYPE_PENDULUM, banlist_limits};
use engine::{DUEL_MODE_MR5, DuelRecord, PlayerOptions, Position};
use oracle::{Message, Prompt, Scripts, Status, parse, respond};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use scripts::CLASSIC_CARDS;
use scripts::testing::Check;

pub use scenario::Scenario;

mod scenario;

const MAX_DECISIONS: u32 = 5_000;
const CONTEXT: usize = 5;
const MSG_WIN: u8 = 5;
const MSG_SELECT_YESNO: u8 = 13;
const MSG_SELECT_TRIBUTE: u8 = 20;
const MSG_SELECT_SUM: u8 = 23;
const MSG_SELECT_UNSELECT_CARD: u8 = 26;
const MSG_MOVE: u8 = 50;
const MSG_POS_CHANGE: u8 = 53;
const MSG_SET: u8 = 54;
const MSG_SWAP: u8 = 55;
const MSG_SUMMONING: u8 = 60;
const MSG_SPSUMMONING: u8 = 62;
const MSG_FLIPSUMMONING: u8 = 64;
const MSG_ATTACK: u8 = 110;
const MSG_CHAINING: u8 = 70;
const MSG_CHAIN_SOLVED: u8 = 73;
const MSG_CHAIN_END: u8 = 74;
const MSG_CHAIN_DISABLED: u8 = 76;
const MSG_EQUIP: u8 = 93;
const MSG_CARD_TARGET: u8 = 96;
const MSG_CARD_HINT: u8 = 160;
const CHINT_DESC_ADD: u8 = 6;
const CHINT_DESC_REMOVE: u8 = 7;
const MSG_PLAYER_HINT: u8 = 165;
const PHINT_DESC_REMOVE: u8 = 7;
const REASON_RELEASE: u32 = 0x2;
const REASON_MATERIAL: u32 = 0x8;
const REASON_SUMMON: u32 = 0x10;
const REASON_REPLACE: u32 = 0x100_0000;
const REASON_BATTLE: u32 = 0x20;
const REASON_RITUAL: u32 = 0x10_0000;
const HINT_REPLAY: u64 = 30;

#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error(transparent)]
    Cards(#[from] CardsError),
    #[error("cannot read vendored data: {0}")]
    Vendor(#[from] std::io::Error),
    #[error(transparent)]
    Oracle(#[from] oracle::DuelError),
    #[error(transparent)]
    Message(#[from] oracle::MessageError),
    #[error(transparent)]
    Engine(#[from] engine::DuelError),
    #[error(transparent)]
    Position(#[from] engine::PositionError),
    #[error("seed {seed}: {error}")]
    Action { seed: u64, error: engine::ActionError },
    #[error("seed {seed}: the oracle awaits a response without sending a prompt")]
    MissingPrompt { seed: u64 },
    #[error("seed {seed}: the oracle ended without MSG_WIN")]
    Ended { seed: u64 },
    #[error("seed {seed}: no response to {prompt:?}")]
    Unanswerable { seed: u64, prompt: Prompt },
    #[error("seed {seed}: no MSG_WIN within {MAX_DECISIONS} decisions")]
    DecisionCap { seed: u64 },
    #[error("seed {seed}: an effect reached the unported call site {site}")]
    Unported { seed: u64, site: &'static str },
    #[error("{0}")]
    Mismatch(Box<Mismatch>),
    #[error("seed {seed}: {message}")]
    Probe { seed: u64, message: String },
}

// The first message the engines disagree on, at `index` in the stream: the last messages both wrote, then each
// engine's messages from the difference on.
#[derive(Debug)]
pub struct Mismatch {
    pub seed: u64,
    pub index: usize,
    pub preceding: Vec<Vec<u8>>,
    pub oracle: Vec<Vec<u8>>,
    pub engine: Vec<Vec<u8>>,
}

// Message types by number, from ocgapi_constants.h.
const MESSAGE_NAMES: &str = concat!(
    "1 RETRY 2 HINT 3 WAITING 4 START 5 WIN 6 UPDATE_DATA 7 UPDATE_CARD 8 REQUEST_DECK 10 SELECT_BATTLECMD 11 ",
    "SELECT_IDLECMD 12 SELECT_EFFECTYN 13 SELECT_YESNO 14 SELECT_OPTION 15 SELECT_CARD 16 SELECT_CHAIN 18 ",
    "SELECT_PLACE 19 SELECT_POSITION 20 SELECT_TRIBUTE 21 SORT_CHAIN 22 SELECT_COUNTER 23 SELECT_SUM 24 ",
    "SELECT_DISFIELD 25 SORT_CARD 26 SELECT_UNSELECT_CARD 30 CONFIRM_DECKTOP 31 CONFIRM_CARDS 32 SHUFFLE_DECK 33 ",
    "SHUFFLE_HAND 34 REFRESH_DECK 35 SWAP_GRAVE_DECK 36 SHUFFLE_SET_CARD 37 REVERSE_DECK 38 DECK_TOP 39 ",
    "SHUFFLE_EXTRA 40 NEW_TURN 41 NEW_PHASE 42 CONFIRM_EXTRATOP 50 MOVE 53 POS_CHANGE 54 SET 55 SWAP 56 ",
    "FIELD_DISABLED 60 SUMMONING 61 SUMMONED 62 SPSUMMONING 63 SPSUMMONED 64 FLIPSUMMONING 65 FLIPSUMMONED 70 ",
    "CHAINING 71 CHAINED 72 CHAIN_SOLVING 73 CHAIN_SOLVED 74 CHAIN_END 75 CHAIN_NEGATED 76 CHAIN_DISABLED 80 ",
    "CARD_SELECTED 81 RANDOM_SELECTED 83 BECOME_TARGET 90 DRAW 91 DAMAGE 92 RECOVER 93 EQUIP 94 LPUPDATE 95 ",
    "UNEQUIP 96 CARD_TARGET 97 CANCEL_TARGET 100 PAY_LPCOST 101 ADD_COUNTER 102 REMOVE_COUNTER 110 ATTACK 111 ",
    "BATTLE 112 ATTACK_DISABLED 113 DAMAGE_STEP_START 114 DAMAGE_STEP_END 120 MISSED_EFFECT 121 BE_CHAIN_TARGET ",
    "122 CREATE_RELATION 123 RELEASE_RELATION 130 TOSS_COIN 131 TOSS_DICE 132 ROCK_PAPER_SCISSORS 133 HAND_RES 140 ",
    "ANNOUNCE_RACE 141 ANNOUNCE_ATTRIB 142 ANNOUNCE_CARD 143 ANNOUNCE_NUMBER 160 CARD_HINT 161 TAG_SWAP 162 ",
    "RELOAD_FIELD 163 AI_NAME 164 SHOW_HINT 165 PLAYER_HINT 170 MATCH_KILL 180 CUSTOM_MSG 190 REMOVE_CARDS",
);

pub fn message_name(kind: u8) -> &'static str {
    let mut words = MESSAGE_NAMES.split(' ');
    std::iter::from_fn(|| Some((words.next()?, words.next()?)))
        .find(|&(number, _)| number.parse() == Ok(kind))
        .map_or("?", |(_, name)| name)
}

pub fn describe(message: &[u8]) -> String {
    match message {
        [kind, body @ ..] => format!("MSG_{} {body:02x?}", message_name(*kind)),
        [] => "empty".to_owned(),
    }
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "seed {}, message {}", self.seed, self.index)?;
        for (label, messages) in [("both", &self.preceding), ("oracle", &self.oracle), ("engine", &self.engine)] {
            writeln!(f, "  {label}:")?;
            messages.iter().try_for_each(|message| writeln!(f, "    {}", describe(message)))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub messages: usize,
    pub decisions: u32,
    pub winner: u8,
    pub reason: u8,
    pub tally: Tally,
    pub record: DuelRecord,
    // Every message of the duel, if the vendor keeps them.
    pub stream: Vec<Vec<u8>>,
}

// How often a duel's messages show the actions phases 1b and 1c added, and the chain links that resolved per card
// code and effect description.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tally {
    pub summons: u32,
    pub sets: u32,
    pub flip_summons: u32,
    pub position_changes: u32,
    pub tributes: u32,
    pub tribute_prompts: u32,
    pub unselect_prompts: u32,
    // MSG_SELECT_SUM prompts for an exact sum and for one reaching the target.
    pub sum_prompts: [u32; 2],
    pub attacks: u32,
    pub direct_attacks: u32,
    pub battle_destructions: u32,
    pub replays: u32,
    pub effects: BTreeMap<(u32, u64), u32>,
    // Special Summons per card code.
    pub special_summons: BTreeMap<u32, u32>,
    // Moves of Ritual Summon materials per card code.
    pub ritual_materials: BTreeMap<u32, u32>,
    // Cards moved as material per card code and the location they left.
    pub materials: BTreeMap<(u32, u8), u32>,
    // Hints added to cards (MSG_CARD_HINT with CHINT_DESC_ADD) per description and card code.
    pub card_hints: BTreeMap<(u64, u32), u32>,
    // Messages per type and the code of the card they are about (see `card_of`), 0 for none.
    pub messages: BTreeMap<(u8, u32), u32>,
    // The code last seen at each (controller, location, sequence), for messages that give a card by its place.
    places: BTreeMap<(u8, u8, u32), u32>,
    // Normal Summons and Sets per card code and number of tributes released for them.
    pub summon_tributes: BTreeMap<(u32, u32), u32>,
    pub set_tributes: BTreeMap<(u32, u32), u32>,
    // Cards destroyed in place of others (MSG_MOVE with REASON_REPLACE) per card code.
    pub replaced: BTreeMap<u32, u32>,
    tributes_pending: u32,
    pub chain_disabled: u32,
    // The card code and effect description of the current chain's links.
    chain: [(u32, u64); 16],
}

impl Tally {
    // The resolved chain links of a card's effects.
    pub fn resolutions(&self, code: u32) -> u32 {
        self.effects.range((code, 0)..=(code, u64::MAX)).map(|(_, count)| count).sum()
    }

    fn count_chain(&mut self, message: &[u8]) {
        match message {
            [MSG_CHAINING, a, b, c, d, .., count, _, _, _] => {
                let description = message
                    .len()
                    .checked_sub(12)
                    .and_then(|start| message.get(start..start + 8))
                    .and_then(|bytes| bytes.try_into().ok())
                    .map_or(0, u64::from_ne_bytes);
                if let Some(link) = self.chain.get_mut(usize::from(*count).wrapping_sub(1)) {
                    *link = (u32::from_ne_bytes([*a, *b, *c, *d]), description);
                }
            }
            [MSG_CHAIN_SOLVED, count] => {
                if let Some(&link) = self.chain.get(usize::from(*count).wrapping_sub(1)) {
                    *self.effects.entry(link).or_default() += 1;
                }
            }
            [MSG_CHAIN_DISABLED, ..] => self.chain_disabled += 1,
            [MSG_CHAIN_END] => self.chain = [(0, 0); 16],
            _ => {}
        }
    }

    // Records where the messages that give a card's code put it: (offset of the code, offset of its location_info).
    fn track(&mut self, message: &[u8]) {
        let moves: &[(usize, usize)] = match message.first() {
            Some(&MSG_MOVE) => &[(1, 15)],
            Some(&(MSG_SUMMONING | MSG_SPSUMMONING | MSG_FLIPSUMMONING | MSG_SET)) => &[(1, 5)],
            Some(&MSG_SWAP) => &[(1, 19), (15, 5)],
            _ => &[],
        };
        for &(code, place) in moves {
            if let (Some(code), Some(place)) = (read_u32(message, code), read_place(message, place)) {
                self.places.insert(place, code);
            }
        }
    }

    // The code of the card a message is about: the first card of MSG_EQUIP and MSG_CARD_TARGET, the hinted card, or
    // the card a message names by code first; 0 for none or a card in an unknown place.
    fn card_of(&self, message: &[u8]) -> u32 {
        let code = match message.first() {
            Some(&(MSG_EQUIP | MSG_CARD_TARGET | MSG_CARD_HINT)) => {
                read_place(message, 1).and_then(|place| self.places.get(&place).copied())
            }
            Some(
                &(MSG_MOVE | MSG_POS_CHANGE | MSG_SET | MSG_SWAP | MSG_SUMMONING | MSG_SPSUMMONING | MSG_FLIPSUMMONING | MSG_CHAINING),
            ) => read_u32(message, 1),
            _ => None,
        };
        code.unwrap_or(0)
    }

    fn count(&mut self, message: &[u8]) {
        self.count_chain(message);
        if let [MSG_SELECT_SUM, _, overflow, ..] = message
            && let Some(count) = self.sum_prompts.get_mut(usize::from(*overflow))
        {
            *count += 1;
        }
        self.track(message);
        if let Some(&kind) = message.first() {
            *self.messages.entry((kind, self.card_of(message))).or_default() += 1;
        }
        if let [MSG_MOVE, a, b, c, d, .., e, f, g, h] = message
            && u32::from_ne_bytes([*e, *f, *g, *h]) & REASON_REPLACE != 0
        {
            *self.replaced.entry(u32::from_ne_bytes([*a, *b, *c, *d])).or_default() += 1;
        }
        match message {
            [MSG_MOVE, .., a, b, c, d] if u32::from_ne_bytes([*a, *b, *c, *d]) & REASON_SUMMON != 0 => self.tributes_pending += 1,
            [kind @ (MSG_SUMMONING | MSG_SET), a, b, c, d, ..] => {
                let code = u32::from_ne_bytes([*a, *b, *c, *d]);
                let tributes = if *kind == MSG_SET {
                    &mut self.set_tributes
                } else {
                    &mut self.summon_tributes
                };
                *tributes.entry((code, self.tributes_pending)).or_default() += 1;
                self.tributes_pending = 0;
            }
            _ => {}
        }
        if let [MSG_SPSUMMONING, a, b, c, d, ..] = message {
            *self.special_summons.entry(u32::from_ne_bytes([*a, *b, *c, *d])).or_default() += 1;
        }
        let reason = |bytes: [&u8; 4]| u32::from_ne_bytes(bytes.map(|byte| *byte));
        if let [MSG_MOVE, a, b, c, d, .., r0, r1, r2, r3] = message
            && reason([r0, r1, r2, r3]) & REASON_RITUAL != 0
        {
            *self.ritual_materials.entry(u32::from_ne_bytes([*a, *b, *c, *d])).or_default() += 1;
        }
        if let [MSG_CARD_HINT, _, _, _, _, _, _, _, _, _, _, CHINT_DESC_ADD, description @ ..] = message
            && let Ok(description) = <[u8; 8]>::try_from(description)
        {
            let card = self.card_of(message);
            *self.card_hints.entry((u64::from_ne_bytes(description), card)).or_default() += 1;
        }
        // The code, the previous location_info (controller, location, sequence, position), the current one and the reason.
        if let [MSG_MOVE, a, b, c, d, _, location, .., e, f, g, h] = message
            && message.len() == 29
            && reason([e, f, g, h]) & REASON_MATERIAL != 0
        {
            *self.materials.entry((u32::from_ne_bytes([*a, *b, *c, *d]), *location)).or_default() += 1;
        }
        if message.first() == Some(&MSG_ATTACK) {
            self.attacks += 1;
        }
        let counter = match message {
            // The target's location_info: controller, location 0 for a direct attack, sequence and position.
            [MSG_ATTACK, .., _, 0, _, _, _, _, _, _, _, _] => &mut self.direct_attacks,
            [MSG_SELECT_YESNO, _, description @ ..] if description == HINT_REPLAY.to_ne_bytes() => &mut self.replays,
            [MSG_SUMMONING, ..] => &mut self.summons,
            [MSG_SET, ..] => &mut self.sets,
            [MSG_FLIPSUMMONING, ..] => &mut self.flip_summons,
            [MSG_POS_CHANGE, ..] => &mut self.position_changes,
            [MSG_SELECT_TRIBUTE, ..] => &mut self.tribute_prompts,
            [MSG_SELECT_UNSELECT_CARD, ..] => &mut self.unselect_prompts,
            [MSG_MOVE, .., a, b, c, d] if reason([a, b, c, d]) & REASON_RELEASE != 0 => &mut self.tributes,
            [MSG_MOVE, .., a, b, c, d] if reason([a, b, c, d]) & REASON_BATTLE != 0 => &mut self.battle_destructions,
            _ => return,
        };
        *counter += 1;
    }
}

impl std::ops::Add for Tally {
    type Output = Tally;

    fn add(self, other: Tally) -> Tally {
        Tally {
            summons: self.summons + other.summons,
            sets: self.sets + other.sets,
            flip_summons: self.flip_summons + other.flip_summons,
            position_changes: self.position_changes + other.position_changes,
            tributes: self.tributes + other.tributes,
            tribute_prompts: self.tribute_prompts + other.tribute_prompts,
            unselect_prompts: self.unselect_prompts + other.unselect_prompts,
            sum_prompts: {
                let ([exact, overflow], [other_exact, other_overflow]) = (self.sum_prompts, other.sum_prompts);
                [exact + other_exact, overflow + other_overflow]
            },
            attacks: self.attacks + other.attacks,
            direct_attacks: self.direct_attacks + other.direct_attacks,
            battle_destructions: self.battle_destructions + other.battle_destructions,
            replays: self.replays + other.replays,
            effects: merge_counts(self.effects, other.effects),
            special_summons: merge_counts(self.special_summons, other.special_summons),
            ritual_materials: merge_counts(self.ritual_materials, other.ritual_materials),
            materials: merge_counts(self.materials, other.materials),
            card_hints: merge_counts(self.card_hints, other.card_hints),
            messages: merge_counts(self.messages, other.messages),
            summon_tributes: merge_counts(self.summon_tributes, other.summon_tributes),
            set_tributes: merge_counts(self.set_tributes, other.set_tributes),
            replaced: merge_counts(self.replaced, other.replaced),
            places: BTreeMap::new(),
            tributes_pending: 0,
            chain_disabled: self.chain_disabled + other.chain_disabled,
            chain: [(0, 0); 16],
        }
    }
}

fn read_u32(message: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_ne_bytes(message.get(at..at + 4)?.try_into().ok()?))
}

// The controller, location and sequence of the location_info at `at`.
fn read_place(message: &[u8], at: usize) -> Option<(u8, u8, u32)> {
    Some((*message.get(at)?, *message.get(at + 1)?, read_u32(message, at + 2)?))
}

fn merge_counts<K: Ord>(counts: BTreeMap<K, u32>, other: BTreeMap<K, u32>) -> BTreeMap<K, u32> {
    other.into_iter().fold(counts, |mut counts, (key, count)| {
        *counts.entry(key).or_default() += count;
        counts
    })
}

pub struct Vendor {
    pub cards: CardDatabase,
    pub scripts: Scripts,
    pub pool: Pool,
    pub effect_cards: Vec<u32>,
    // A fixed deck both players play, shuffled per duel, in place of random pool decks.
    pub deck: Option<Deck>,
    // The cards a scenario places at random besides the card under test and its support.
    pub scenario_others: Vec<u32>,
    // Messages shown before and from a divergence.
    pub context: usize,
    pub keep_stream: bool,
}

impl Vendor {
    // One of the constructors below on the vendored data, leaked for the 'static lifetime the engine takes.
    pub fn leak(kind: fn(&Path) -> Result<Self, DiffError>) -> Result<&'static Self, DiffError> {
        let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
        Ok(Box::leak(Box::new(kind(&vendor)?)))
    }

    // The Advanced pool reduced to Normal Monsters, which have no scripts; Normal Pendulums do.
    pub fn normal_monsters(vendor: &Path) -> Result<Self, DiffError> {
        let cards = CardDatabase::load(&vendor.join("BabelCDB"))?;
        let mut pool = Pool::advanced(
            &cards,
            banlist_limits(&std::fs::read_to_string(vendor.join("LFLists/0TCG.lflist.conf"))?),
        );
        pool.retain(|code| {
            cards
                .get(code)
                .is_some_and(|card| card.card_type & (TYPE_NORMAL | TYPE_PENDULUM) == TYPE_NORMAL)
        });
        Ok(Self {
            scripts: Scripts::load(&vendor.join("CardScripts"))?,
            cards,
            pool,
            effect_cards: Vec::new(),
            deck: None,
            scenario_others: Vec::new(),
            context: CONTEXT,
            keep_stream: false,
        })
    }

    // Normal Monsters and zero to two copies of each card of phase 2a.
    pub fn classic_effects(vendor: &Path) -> Result<Self, DiffError> {
        Ok(Self {
            effect_cards: CLASSIC_CARDS.to_vec(),
            ..Self::normal_monsters(vendor)?
        })
    }

    // Mirror matches of Elfnote, 1st place YCS Montreal 2026: the main and Extra Deck, without the side deck.
    pub fn elfnote(vendor: &Path) -> Result<Self, DiffError> {
        let base = Self::normal_monsters(vendor)?;
        let deck = Deck::parse(include_str!("../elfnote-ycs-montreal-2026.deck"), &base.cards);
        Ok(Self { deck: Some(deck), ..base })
    }

    // Elfnote mirror matches starting from a scenario. The other cards placed come from the Elfnote deck and the classic
    // cards, which never change, so that new ports leave every scenario as it was.
    pub fn scenarios(vendor: &Path) -> Result<Self, DiffError> {
        let base = Self::elfnote(vendor)?;
        let deck = base.deck.iter().flat_map(|deck| deck.main.iter().chain(&deck.extra));
        let others: BTreeSet<u32> = deck.chain(&CLASSIC_CARDS).copied().collect();
        Ok(Self {
            scenario_others: others.into_iter().collect(),
            ..base
        })
    }

    fn deck(&self, rng: &mut StdRng) -> Deck {
        match &self.deck {
            Some(deck) => deck.shuffled(rng),
            None => self.pool.random_deck_with(&self.effect_cards, rng),
        }
    }
}

// Plays one seeded duel in the oracle and the engine with the same decks and random responses and compares every
// message up to MSG_WIN.
pub fn diff_duel(vendor: &'static Vendor, seed: u64) -> Result<Report, DiffError> {
    play(vendor, seed, None, None, &mut |_, _| Ok(()))
}

// Scenario `index` of `card`, seeded by both alone, so that adding ports changes no other card's scenarios.
pub fn diff_scenario(vendor: &'static Vendor, card: u32, index: u32) -> Result<Report, DiffError> {
    diff_scenario_probed(vendor, card, index, &mut |_, _| Ok(()))
}

// A check of both engines at every prompt, once their messages agree.
pub type Probe<'a> = dyn FnMut(&mut oracle::Duel<'_>, &engine::Duel) -> Result<(), String> + 'a;

pub fn diff_duel_probed(vendor: &'static Vendor, seed: u64, probe: &mut Probe) -> Result<Report, DiffError> {
    play(vendor, seed, None, None, probe)
}

pub fn diff_scenario_probed(vendor: &'static Vendor, card: u32, index: u32, probe: &mut Probe) -> Result<Report, DiffError> {
    play(vendor, scenario_seed(card, index), Some(card), None, probe)
}

fn scenario_seed(card: u32, index: u32) -> u64 {
    u64::from(card) << 32 | u64::from(index)
}

// What scenario `index` of `card` places.
pub fn scenario(vendor: &Vendor, card: u32, index: u32) -> Option<Scenario> {
    seeded(vendor, scenario_seed(card, index), Some(card)).2
}

fn seeded(vendor: &Vendor, seed: u64, card: Option<u32>) -> (StdRng, [u64; 4], Option<Scenario>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let duel_seed: [u64; 4] = rng.random();
    let scenario = card.map(|card| Scenario::generate(card, &vendor.scenario_others, &vendor.cards, &mut rng));
    (rng, duel_seed, scenario)
}

// A duel both engines set up from `position` alone, which must start the duel (see scenario::lua).
pub fn diff_position(vendor: &'static Vendor, seed: u64, position: &Position) -> Result<Report, DiffError> {
    play(vendor, seed, None, Some(position), &mut |_, _| Ok(()))
}

// Both engines start from one position: the decks, the scenario of `card` if any on top, or `position` if given.
fn play(
    vendor: &'static Vendor,
    seed: u64,
    card: Option<u32>,
    position: Option<&Position>,
    probe: &mut Probe,
) -> Result<Report, DiffError> {
    let (mut rng, duel_seed, scenario) = seeded(vendor, seed, card);
    let position = match position {
        Some(position) => position.clone(),
        None => {
            let lp = scenario.as_ref().map_or([8000; 2], |scenario| scenario.lp);
            let players = lp.map(|starting_lp| PlayerOptions {
                starting_lp,
                ..PlayerOptions::default()
            });
            let mut position = Position::new(DUEL_MODE_MR5, players);
            for team in 0..2 {
                position.add_deck(team, &vendor.deck(&mut rng));
            }
            position
                .cards
                .extend(scenario.iter().flat_map(|scenario| scenario.cards.iter().cloned()));
            position
        }
    };
    let mut oracle = oracle::Duel::new(&vendor.cards, &vendor.scripts, duel_seed)?;
    oracle.run_lua(&scenario::lua(&position))?;
    let mut engine = engine::Duel::from_position(&position, &vendor.cards, &scripts::LIBRARY, duel_seed)?;
    engine.set_trace(true);
    let mut record = DuelRecord {
        seed: duel_seed,
        position,
        actions: Vec::new(),
    };
    oracle.start();
    engine.process();
    let (mut compared, mut decisions, mut tally): (Vec<Vec<u8>>, u32, Tally) = (Vec::new(), 0, Tally::default());
    for (place, code) in scenario.iter().flat_map(Scenario::places) {
        tally.places.entry(place).or_insert(code);
    }
    loop {
        let mut expected = Vec::new();
        let status = loop {
            let status = oracle.process()?;
            expected.extend(oracle.messages().map(<[u8]>::to_vec));
            if status != Status::Continue {
                break status;
            }
        };
        sort_hint_removals(&mut expected);
        if let Some(site) = engine.unported_site() {
            return Err(DiffError::Unported { seed, site });
        }
        let trace = engine.take_trace();
        let mut actual = frames(&trace);
        sort_hint_removals(&mut actual);
        let mismatch = |compared: &[Vec<u8>], from: usize| {
            let context = if vendor.keep_stream { usize::MAX } else { vendor.context };
            DiffError::Mismatch(Box::new(Mismatch {
                seed,
                index: compared.len(),
                preceding: compared.iter().skip(compared.len().saturating_sub(context)).cloned().collect(),
                oracle: expected.iter().skip(from).take(context).cloned().collect(),
                engine: actual.iter().skip(from).take(context).map(|message| message.to_vec()).collect(),
            }))
        };
        for (index, message) in expected.iter().enumerate() {
            if actual.get(index).copied() != Some(message.as_slice()) {
                return Err(mismatch(&compared, index));
            }
            compared.push(message.clone());
            tally.count(message);
            if let [MSG_WIN, winner, reason, ..] = message.as_slice() {
                return Ok(Report {
                    messages: compared.len(),
                    decisions,
                    winner: *winner,
                    reason: *reason,
                    tally,
                    record,
                    stream: if vendor.keep_stream { compared } else { Vec::new() },
                });
            }
        }
        if actual.len() > expected.len() {
            return Err(mismatch(&compared, expected.len()));
        }
        match status {
            Status::End => return Err(DiffError::Ended { seed }),
            Status::Awaiting if decisions == MAX_DECISIONS => return Err(DiffError::DecisionCap { seed }),
            _ => {}
        }
        let Some(Message::Prompt(prompt)) = expected.last().map(|message| parse(message)).transpose()? else {
            return Err(DiffError::MissingPrompt { seed });
        };
        probe(&mut oracle, &engine).map_err(|message| DiffError::Probe { seed, message })?;
        let response = respond(&prompt, &vendor.cards, &mut rng).ok_or(DiffError::Unanswerable { seed, prompt })?;
        oracle.set_response(&response);
        let actions = engine
            .decode_response(&response)
            .map_err(|error| DiffError::Action { seed, error })?;
        for action in actions {
            engine.apply(action).map_err(|error| DiffError::Action { seed, error })?;
            record.actions.push(action);
        }
        decisions += 1;
    }
}

// The core removes hint effects iterating unordered sets of effect pointers (field::effects.cheff and pheff), and
// re-adds the hints of a shuffled hand iterating each card's unordered indexer, so the order of consecutive
// MSG_CARD_HINT and MSG_PLAYER_HINT removals, and of consecutive MSG_CARD_HINT additions, varies between runs of the
// oracle; they compare as a set.
fn sort_hint_removals<T: AsRef<[u8]> + Ord>(messages: &mut [T]) {
    let unordered = |message: &T| match message.as_ref() {
        [MSG_CARD_HINT, _, _, _, _, _, _, _, _, _, _, CHINT_DESC_REMOVE, ..] | [MSG_PLAYER_HINT, _, PHINT_DESC_REMOVE, ..] => {
            Some(CHINT_DESC_REMOVE)
        }
        [MSG_CARD_HINT, _, _, _, _, _, _, _, _, _, _, CHINT_DESC_ADD, ..] => Some(CHINT_DESC_ADD),
        _ => None,
    };
    for run in messages.chunk_by_mut(|a, b| unordered(a).is_some() && unordered(a) == unordered(b)) {
        run.sort();
    }
}

// Splits a trace in the framing of OCG_DuelGetMessage.
fn frames(mut buffer: &[u8]) -> Vec<&[u8]> {
    std::iter::from_fn(|| {
        let (size, rest) = buffer.split_first_chunk::<4>()?;
        let (message, rest) = rest.split_at_checked(u32::from_ne_bytes(*size) as usize)?;
        buffer = rest;
        Some(message)
    })
    .collect()
}

// Scenarios per card in the default test (about 1000 for the 78 cards of phase 4a) and in the ignored one.
pub const SCENARIOS: u32 = 13;
pub const SCENARIOS_FULL: u32 = 64;

// Scenarios 0..count of each card, in parallel.
pub fn diff_cards(vendor: &'static Vendor, cards: &[u32], count: u32) -> Vec<Vec<Result<Report, DiffError>>> {
    cards
        .par_iter()
        .map(|&card| (0..count).into_par_iter().map(|index| diff_scenario(vendor, card, index)).collect())
        .collect()
}

// How often a card's scenarios show what the check asks for.
pub fn check_count(check: Check, card: u32, tally: &Tally) -> u32 {
    fn get<K: Ord>(counts: &BTreeMap<K, u32>, key: K) -> u32 {
        counts.get(&key).copied().unwrap_or(0)
    }
    match check {
        Check::Resolves => tally.resolutions(card),
        Check::ResolvesEffect(description) => get(&tally.effects, (card, description)),
        Check::SpecialSummoned => get(&tally.special_summons, card),
        Check::SpecialSummonedCode(code) => get(&tally.special_summons, code),
        Check::RitualMaterial => get(&tally.ritual_materials, card),
        Check::MaterialFrom(location) => get(&tally.materials, (card, location as u8)),
        Check::CardHint(description) => get(&tally.card_hints, (description, card)),
        Check::Message(kind) => get(&tally.messages, (kind, card)) + get(&tally.messages, (kind, 0)),
        Check::TributeSummoned(tributes) => get(&tally.summon_tributes, (card, tributes)),
        Check::TributeSet(tributes) => get(&tally.set_tributes, (card, tributes)),
        Check::Replaced => get(&tally.replaced, card),
        Check::Report(check) => check_count(*check, card, tally),
    }
}

// A card's scenarios written out for the card command and the scenario tests, and whether every one diffs clean and
// every check passes, both over the first SCENARIOS (the default test) and over all of them.
pub fn summarize(card: u32, results: &[Result<Report, DiffError>]) -> (bool, String) {
    let tally = |count: usize| {
        let reports = results.iter().take(count).flatten();
        reports.fold(Tally::default(), |tally, report| tally + report.tally.clone())
    };
    let first = results.len().min(SCENARIOS as usize);
    let (short, full) = (tally(first), tally(results.len()));
    let failures: Vec<(usize, &DiffError)> = results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| Some((index, result.as_ref().err()?)))
        .collect();
    let mut passed = failures.is_empty();
    let mut out = String::new();
    let checks = scripts::card(card).map_or(&[Check::Resolves][..], |script| script.checks);
    for &check in checks {
        let [in_first, in_all] = [&short, &full].map(|tally| check_count(check, card, tally));
        let ok = matches!(check, Check::Report(_)) || (in_first > 0 && in_all > 0);
        passed &= ok;
        let status = if ok { "" } else { " FAIL" };
        let hits = results.iter().enumerate().filter_map(|(index, result)| {
            result
                .as_ref()
                .ok()
                .filter(|report| check_count(check, card, &report.tally) > 0)
                .map(|_| index.to_string())
        });
        let hits = hits.take(8).collect::<Vec<_>>().join(" ");
        writeln!(
            out,
            "  {check:?}: {in_first} in {first}, {in_all} in {}{status} (scenarios {hits})",
            results.len()
        )
        .ok();
    }
    let resolved = full.effects.range((card, 0)..=(card, u64::MAX)).map(|(&(_, description), count)| {
        match description.checked_sub(u64::from(card) << 20) {
            Some(index) if index < 16 => format!("#{index} {count}"),
            _ => format!("{description:#x} {count}"),
        }
    });
    writeln!(out, "  resolved in {}: {}", results.len(), resolved.collect::<Vec<_>>().join(", ")).ok();
    if let Some((index, error)) = failures.first() {
        let others: Vec<String> = failures.iter().skip(1).map(|(index, _)| index.to_string()).collect();
        writeln!(out, "  scenario {index}: {}", error.to_string().trim_end()).ok();
        if !others.is_empty() {
            writeln!(out, "  also failing: scenarios {}", others.join(", ")).ok();
        }
    }
    let verdict = if passed { "pass" } else { "FAIL" };
    let clean = results.len() - failures.len();
    (
        passed,
        format!("{card} {verdict}: {clean} of {} scenarios clean\n{out}", results.len()),
    )
}
