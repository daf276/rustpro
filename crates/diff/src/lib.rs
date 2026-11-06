use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use carddata::{CardDatabase, CardsError, Deck, Pool, TYPE_NORMAL, TYPE_PENDULUM, banlist_limits};
use engine::{DUEL_MODE_MR5, DuelOptions, PlayerOptions};
use oracle::{Message, Prompt, Scripts, Status, parse, respond};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use scripts::{CARD_SCRIPTS, CLASSIC_CARDS};

pub use scenario::Scenario;

mod scenario;

const MAX_DECISIONS: u32 = 5_000;
const CONTEXT: usize = 5;
const MSG_WIN: u8 = 5;
const MSG_SELECT_YESNO: u8 = 13;
const MSG_SELECT_TRIBUTE: u8 = 20;
const MSG_SELECT_UNSELECT_CARD: u8 = 26;
const MSG_MOVE: u8 = 50;
const MSG_POS_CHANGE: u8 = 53;
const MSG_SET: u8 = 54;
const MSG_SUMMONING: u8 = 60;
const MSG_SPSUMMONING: u8 = 62;
const MSG_FLIPSUMMONING: u8 = 64;
const MSG_ATTACK: u8 = 110;
const MSG_CHAINING: u8 = 70;
const MSG_CHAIN_SOLVED: u8 = 73;
const MSG_CHAIN_END: u8 = 74;
const MSG_CHAIN_DISABLED: u8 = 76;
const MSG_CARD_HINT: u8 = 160;
const CHINT_DESC_ADD: u8 = 6;
const CHINT_DESC_REMOVE: u8 = 7;
const MSG_PLAYER_HINT: u8 = 165;
const PHINT_DESC_REMOVE: u8 = 7;
const REASON_RELEASE: u32 = 0x2;
const REASON_MATERIAL: u32 = 0x8;
const REASON_SUMMON: u32 = 0x10;
const REASON_BATTLE: u32 = 0x20;
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
}

#[derive(Debug)]
pub struct Mismatch {
    pub seed: u64,
    pub index: usize,
    pub oracle: Option<Vec<u8>>,
    pub engine: Option<Vec<u8>>,
    pub preceding: Vec<Vec<u8>>,
}

fn describe(message: Option<&[u8]>) -> String {
    match message {
        Some([id, body @ ..]) => format!("type {id}: {body:02x?}"),
        Some([]) => "empty".to_owned(),
        None => "none".to_owned(),
    }
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "seed {}, message {}:", self.seed, self.index)?;
        writeln!(f, "  oracle {}", describe(self.oracle.as_deref()))?;
        writeln!(f, "  engine {}", describe(self.engine.as_deref()))?;
        writeln!(f, "  preceding:")?;
        self.preceding
            .iter()
            .try_for_each(|message| writeln!(f, "    {}", describe(Some(message))))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub messages: usize,
    pub decisions: u32,
    pub winner: u8,
    pub reason: u8,
    pub tally: Tally,
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
    pub attacks: u32,
    pub direct_attacks: u32,
    pub battle_destructions: u32,
    pub replays: u32,
    pub effects: BTreeMap<(u32, u64), u32>,
    // Special Summons per card code.
    pub special_summons: BTreeMap<u32, u32>,
    // Cards moved as material per card code and the location they left.
    pub materials: BTreeMap<(u32, u8), u32>,
    // Hints added to cards per description (MSG_CARD_HINT with CHINT_DESC_ADD).
    pub card_hints: BTreeMap<u64, u32>,
    // Messages per type.
    pub messages: BTreeMap<u8, u32>,
    // Normal Summons and Sets per card code and number of tributes released for them.
    pub summon_tributes: BTreeMap<(u32, u32), u32>,
    pub set_tributes: BTreeMap<(u32, u32), u32>,
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

    fn count(&mut self, message: &[u8]) {
        self.count_chain(message);
        if let Some(&kind) = message.first() {
            *self.messages.entry(kind).or_default() += 1;
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
        if let [MSG_CARD_HINT, _, _, _, _, _, _, _, _, _, _, CHINT_DESC_ADD, description @ ..] = message
            && let Ok(description) = <[u8; 8]>::try_from(description)
        {
            *self.card_hints.entry(u64::from_ne_bytes(description)).or_default() += 1;
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
            attacks: self.attacks + other.attacks,
            direct_attacks: self.direct_attacks + other.direct_attacks,
            battle_destructions: self.battle_destructions + other.battle_destructions,
            replays: self.replays + other.replays,
            effects: other.effects.into_iter().fold(self.effects, |mut effects, (link, count)| {
                *effects.entry(link).or_default() += count;
                effects
            }),
            special_summons: other
                .special_summons
                .into_iter()
                .fold(self.special_summons, |mut summons, (code, count)| {
                    *summons.entry(code).or_default() += count;
                    summons
                }),
            materials: other.materials.into_iter().fold(self.materials, |mut materials, (key, count)| {
                *materials.entry(key).or_default() += count;
                materials
            }),
            card_hints: other
                .card_hints
                .into_iter()
                .fold(self.card_hints, |mut hints, (description, count)| {
                    *hints.entry(description).or_default() += count;
                    hints
                }),
            messages: other.messages.into_iter().fold(self.messages, |mut messages, (kind, count)| {
                *messages.entry(kind).or_default() += count;
                messages
            }),
            summon_tributes: other
                .summon_tributes
                .into_iter()
                .fold(self.summon_tributes, |mut summons, (key, count)| {
                    *summons.entry(key).or_default() += count;
                    summons
                }),
            set_tributes: other.set_tributes.into_iter().fold(self.set_tributes, |mut sets, (key, count)| {
                *sets.entry(key).or_default() += count;
                sets
            }),
            tributes_pending: 0,
            chain_disabled: self.chain_disabled + other.chain_disabled,
            chain: [(0, 0); 16],
        }
    }
}

pub struct Vendor {
    pub cards: CardDatabase,
    pub scripts: Scripts,
    pub pool: Pool,
    pub effect_cards: Vec<u32>,
    // A fixed deck both players play, shuffled per duel, in place of random pool decks.
    pub deck: Option<Deck>,
    // If not empty, every duel is a scenario for one of these cards in turn.
    pub scenario_cards: Vec<u32>,
}

impl Vendor {
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
            scenario_cards: Vec::new(),
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

    // Elfnote mirror matches starting from a scenario for each ported card in turn.
    pub fn scenarios(vendor: &Path) -> Result<Self, DiffError> {
        Ok(Self {
            scenario_cards: CARD_SCRIPTS.iter().map(|&(code, _)| code).collect(),
            ..Self::elfnote(vendor)?
        })
    }

    // The card a seed's scenario tests.
    pub fn scenario_card(&self, seed: u64) -> Option<u32> {
        let index = seed.checked_rem(self.scenario_cards.len() as u64)?;
        self.scenario_cards.get(index as usize).copied()
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
    let mut rng = StdRng::seed_from_u64(seed);
    let duel_seed: [u64; 4] = rng.random();
    let scenario = vendor
        .scenario_card(seed)
        .map(|card| Scenario::generate(card, &vendor.scenario_cards, &vendor.cards, &mut rng));
    let mut oracle = oracle::Duel::new(&vendor.cards, &vendor.scripts, duel_seed)?;
    let [team1, team2] = scenario
        .as_ref()
        .map_or([8000; 2], |scenario| scenario.lp)
        .map(|starting_lp| PlayerOptions {
            starting_lp,
            ..PlayerOptions::default()
        });
    let mut engine = engine::Duel::new(DuelOptions {
        seed: duel_seed,
        flags: DUEL_MODE_MR5,
        team1,
        team2,
    })?;
    engine.set_trace(true);
    engine.load_library(&scripts::LIBRARY);
    for team in 0..2 {
        let deck = vendor.deck(&mut rng);
        oracle.add_deck(team, &deck);
        engine.add_deck(team, &deck, &vendor.cards)?;
    }
    if let Some(scenario) = &scenario {
        oracle.run_lua(&scenario.lua())?;
        scenario.place(&mut engine, &vendor.cards)?;
    }
    oracle.start();
    engine.start();
    engine.process();
    let (mut compared, mut decisions, mut tally): (Vec<Vec<u8>>, u32, Tally) = (Vec::new(), 0, Tally::default());
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
        let mismatch = |compared: &[Vec<u8>], oracle: Option<&Vec<u8>>, engine: Option<&[u8]>| {
            let preceding = compared.iter().skip(compared.len().saturating_sub(CONTEXT)).cloned().collect();
            DiffError::Mismatch(Box::new(Mismatch {
                seed,
                index: compared.len(),
                oracle: oracle.cloned(),
                engine: engine.map(<[u8]>::to_vec),
                preceding,
            }))
        };
        for (index, message) in expected.iter().enumerate() {
            let engine_message = actual.get(index).copied();
            if engine_message != Some(message.as_slice()) {
                return Err(mismatch(&compared, Some(message), engine_message));
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
                });
            }
        }
        if let Some(&extra) = actual.get(expected.len()) {
            return Err(mismatch(&compared, None, Some(extra)));
        }
        match status {
            Status::End => return Err(DiffError::Ended { seed }),
            Status::Awaiting if decisions == MAX_DECISIONS => return Err(DiffError::DecisionCap { seed }),
            _ => {}
        }
        let Some(Message::Prompt(prompt)) = expected.last().map(|message| parse(message)).transpose()? else {
            return Err(DiffError::MissingPrompt { seed });
        };
        let response = respond(&prompt, &vendor.cards, &mut rng).ok_or(DiffError::Unanswerable { seed, prompt })?;
        oracle.set_response(&response);
        let actions = engine
            .decode_response(&response)
            .map_err(|error| DiffError::Action { seed, error })?;
        for action in actions {
            engine.apply(action).map_err(|error| DiffError::Action { seed, error })?;
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
