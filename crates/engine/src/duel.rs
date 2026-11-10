use carddata::{CardDatabase, Deck};
use serde::{Deserialize, Serialize};

use crate::buffer::{ProgressiveBuffer, Trace};
use crate::card::{Card, CardId, CardSet, Cards, MAX_CARDS, StaticData};
use crate::constants::*;
use crate::effect::{EffectId, Effects, Event, FieldEffect};
use crate::field::{FieldInfo, PerPlayer, PlayerInfo, Processor, Relations, ReturnCards};
use crate::list::List;
use crate::processor::{OcgStatus, Startup};
use crate::rng::Xoshiro256StarStar;
use crate::script::{FusionCheck, Library};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DuelError {
    #[error("the seed must not be all zero")]
    NullSeed,
    #[error("only the first duelist of each team is supported")]
    Duelist,
    #[error("a duel holds at most {MAX_CARDS} cards")]
    TooManyCards,
    #[error("card {0} is not in the database")]
    UnknownCard(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerOptions {
    pub starting_lp: u32,
    pub starting_draw_count: u32,
    pub draw_count_per_turn: u32,
}

impl Default for PlayerOptions {
    fn default() -> Self {
        Self {
            starting_lp: 8000,
            starting_draw_count: 5,
            draw_count_per_turn: 1,
        }
    }
}

// OCG_DuelOptions without the reader callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuelOptions {
    pub seed: [u64; 4],
    pub flags: u64,
    pub team1: PlayerOptions,
    pub team2: PlayerOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewCardInfo {
    pub team: u8,
    pub duelist: u8,
    pub code: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Awaiting,
    End,
}

// duel and field of the core in one value; cloning it at any prompt resumes from there.
#[derive(Debug, Clone)]
pub struct Duel {
    pub(crate) cards: Cards,
    pub(crate) player: PerPlayer<PlayerInfo>,
    pub(crate) infos: FieldInfo,
    // duel::effects, the arena of every effect.
    pub(crate) effects: Effects,
    // field::effects, the containers and collections.
    pub(crate) field_effects: FieldEffect,
    pub(crate) relations: Relations,
    pub(crate) core: Processor,
    pub(crate) returns: ProgressiveBuffer,
    pub(crate) return_cards: ReturnCards,
    pub(crate) nil_event: Event,
    pub(crate) random: Xoshiro256StarStar,
    pub(crate) trace: Trace,
    pub(crate) result: Option<(u8, u8)>,
    pub(crate) library: Option<&'static Library>,
    // The database the duel's cards were read from, which Duel.CreateToken and Duel.AnnounceCard read at runtime.
    pub(crate) database: Option<&'static CardDatabase>,
    // The scripts' own state for this duel (s.xxx tables and aux.GlobalCheck), keyed by script and slot.
    pub(crate) script_state: List<(u64, i64), 16>,
    pub(crate) script_groups: Vec<(u64, CardSet)>,
    // aux.ToResetFuncTable: the functions the turn end runs, in the order the scripts registered them.
    pub(crate) values_reset: Vec<fn(&mut Duel)>,
    // proc_fusion.lua's Fusion.CheckAdditional, a function value the scripts set around a Fusion Summon.
    pub(crate) fusion_check_additional: Option<FusionCheck>,
    // proc_ritual.lua's Ritual.SummoningLevel, which Ritual.Filter sets while a check runs.
    pub(crate) ritual_summoning_level: std::cell::Cell<Option<u32>>,
    // card::assume of the cards duel::assumes lists, as (card, ASSUME_ kind, value): what Card.AssumeProperty fixes until
    // Duel.AssumeReset or the end of the outermost script call. Set in checks that only read.
    pub(crate) assumes: std::cell::RefCell<List<(CardId, u8, u64), 2>>,
    // Effects a check reset on a copy of this duel (check_mut), reset here at the next processor step.
    pub(crate) pending_resets: std::cell::RefCell<List<EffectId, 2>>,
    // card::material_cards of the cards that have materials.
    pub(crate) material_cards: Vec<(CardId, CardSet)>,
    // The cards attached as Xyz materials; card::xyz_materials of a card are those whose overlay_target it is.
    pub(crate) overlay_cards: CardSet,
    // card::counters: card, counter type, the permanent count and the count a negation removes.
    pub(crate) counters: Vec<(CardId, u16, [u16; 2])>,
    // card::effect_target_cards as (owner, target) pairs; effect_target_owner is the same pairs read the other way.
    pub(crate) card_targets: Vec<(CardId, CardId)>,
    // card::relations: the card, the card it relates to and the reset flags that end it.
    pub(crate) card_relations: Vec<(CardId, CardId, u32)>,
    // Card.announced_cards: per attacker, the fieldid_r of each target it announced an attack on (0 for a direct attack) and how often.
    pub(crate) announced_cards: Vec<(CardId, u32, u8)>,
    // Card.indestructable_effects: one entry per time an EFFECT_INDESTRUCTABLE_COUNT effect without a count limit was spent on a card.
    pub(crate) indestructable_effects: Vec<(CardId, EffectId)>,
    // Card.battled_cards: per card, the fieldid_r of each card it battled (0 for a direct attack).
    pub(crate) battled_cards: Vec<(CardId, u32)>,
    // The cards equipped to a monster; card::equiping_cards of a card are those whose equiping_target it is.
    pub(crate) equip_cards: CardSet,
    // card::immune_effect as (card, effect) pairs: refreshed only by refresh_disable_status, so an effect registered
    // since is not in it.
    pub(crate) immune_effects: List<(CardId, EffectId), 16>,
    // The first unported call site an effect reached, set in checks that only read.
    pub(crate) unported: std::cell::Cell<Option<&'static str>>,
    // The card and destination a cost check sets as sendto_param.location while it asks the redirects (see cost_destination);
    // None between calls.
    pub(crate) destination_probe: std::cell::Cell<Option<(CardId, u8)>>,
}

impl Duel {
    pub fn new(options: DuelOptions) -> Result<Self, DuelError> {
        if options.seed == [0; 4] {
            return Err(DuelError::NullSeed);
        }
        let player = |team: PlayerOptions| PlayerInfo {
            lp: team.starting_lp as i32,
            start_count: team.starting_draw_count as i32,
            draw_count: team.draw_count_per_turn as i32,
            ..PlayerInfo::default()
        };
        let mut cards = Cards::default();
        // field::temp_card, card 0: the owner of global effects.
        cards.push(Card::new(StaticData::default()));
        Ok(Self {
            cards,
            player: PerPlayer([player(options.team1), player(options.team2)]),
            infos: FieldInfo::default(),
            effects: Effects::default(),
            field_effects: FieldEffect::default(),
            relations: Relations::default(),
            core: Processor::new(options.flags),
            returns: ProgressiveBuffer::default(),
            return_cards: ReturnCards::default(),
            nil_event: Event {
                event_player: PLAYER_NONE,
                reason_player: PLAYER_NONE,
                ..Event::default()
            },
            random: Xoshiro256StarStar::new(options.seed),
            trace: Trace::default(),
            result: None,
            library: None,
            database: None,
            script_state: List::default(),
            script_groups: Vec::new(),
            values_reset: Vec::new(),
            fusion_check_additional: None,
            ritual_summoning_level: Default::default(),
            assumes: Default::default(),
            pending_resets: Default::default(),
            material_cards: Vec::new(),
            overlay_cards: CardSet::default(),
            counters: Vec::new(),
            card_targets: Vec::new(),
            card_relations: Vec::new(),
            announced_cards: Vec::new(),
            indestructable_effects: Vec::new(),
            battled_cards: Vec::new(),
            immune_effects: List::default(),
            equip_cards: CardSet::default(),
            unported: std::cell::Cell::new(None),
            destination_probe: std::cell::Cell::new(None),
        })
    }

    // What the host's OCG_LoadScript of utility.lua does (its global effects), and the scripts new cards load.
    pub fn load_library(&mut self, library: &'static Library) {
        self.library = Some(library);
        (library.load)(self);
    }

    // Collects messages in the framing of OCG_DuelGetMessage; off by default, so simulation never writes them.
    pub fn set_trace(&mut self, enabled: bool) {
        *self.trace.buffer.get_mut() = enabled.then(Vec::new);
    }

    pub fn take_trace(&mut self) -> Vec<u8> {
        self.trace.buffer.get_mut().as_mut().map(std::mem::take).unwrap_or_default()
    }

    // OCG_DuelNewCard: sequence 0 in the deck is the top, so the last card added is drawn first.
    pub fn new_card(&mut self, info: NewCardInfo, database: &'static CardDatabase) -> Result<(), DuelError> {
        if info.loc.count_ones() > 1 {
            return Ok(());
        }
        if info.duelist != 0 {
            return Err(DuelError::Duelist);
        }
        if !self.is_location_useable(info.con, info.loc as u16, info.seq) {
            return Ok(());
        }
        let card = self.create_card(info.code, database)?;
        self.cards[card].owner = info.team;
        self.add_card(info.con, card, info.loc as u8, info.seq as u8, false);
        self.place(card, info.pos, info.loc as u8 == LOCATION_MZONE);
        Ok(())
    }

    // Debug.AddCard, which puzzles use: unlike OCG_DuelNewCard it places face-up Pendulums in the Extra Deck and the
    // Pendulum Zones, and a card placed on an occupied field zone becomes an Xyz material of the card there.
    // `info.team` is the owner and `info.con` the controller. Returns the card, or None if there was no room.
    pub fn debug_add_card(&mut self, info: NewCardInfo, proc: bool, database: &'static CardDatabase) -> Result<Option<CardId>, DuelError> {
        let location = info.loc as u16;
        if !self.is_location_useable(info.con, location, info.seq) {
            let Some(target) = self.field_card(info.con, location, info.seq) else {
                return Ok(None);
            };
            let card = self.create_card(info.code, database)?;
            self.cards[card].owner = info.team;
            self.xyz_add(target, card);
            if proc {
                self.cards[card].set_status(STATUS_PROC_COMPLETE, true);
            }
            return Ok(Some(card));
        }
        let card = self.create_card(info.code, database)?;
        self.cards[card].owner = info.team;
        let pendulum = self.cards[card].data.card_type & TYPE_PENDULUM != 0;
        let position = if location == u16::from(LOCATION_EXTRA) && (info.pos == 0 || !pendulum) {
            POS_FACEDOWN_DEFENSE
        } else {
            info.pos
        };
        self.cards[card].sendto_param.position = position as u8;
        let three_columns = u32::from(self.is_flag(DUEL_3_COLUMNS_FIELD));
        let (location, sequence, pzone) = match location {
            LOCATION_PZONE => (LOCATION_SZONE, self.get_pzone_index(info.seq), true),
            LOCATION_FZONE => (LOCATION_SZONE, 5, false),
            LOCATION_STZONE => (LOCATION_SZONE, info.seq + three_columns, false),
            LOCATION_MMZONE => (LOCATION_MZONE, info.seq + three_columns, false),
            LOCATION_EMZONE => (LOCATION_MZONE, info.seq + 5, false),
            _ => (location as u8, info.seq, false),
        };
        self.add_card(info.con, card, location, sequence as u8, pzone);
        self.place(card, position, proc);
        Ok(Some(card))
    }

    // pduel->new_card: the card object with its script's initial effects, not yet in any location.
    pub(crate) fn create_card(&mut self, code: u32, database: &'static CardDatabase) -> Result<CardId, DuelError> {
        self.database = Some(database);
        // The core takes an unknown code with blank data; here it is an error, since the data cannot carry the code.
        let data = match code {
            0 => StaticData::default(),
            code => database.get(code).ok_or(DuelError::UnknownCard(code))?.into(),
        };
        let card = self.cards.push(Card::new(data)).ok_or(DuelError::TooManyCards)?;
        self.register_card(card);
        Ok(card)
    }

    fn place(&mut self, card: CardId, position: u32, proc: bool) {
        self.cards[card].current.position = position;
        if self.cards[card].current.location & LOCATION_ONFIELD == 0 || position & POS_FACEUP != 0 {
            self.enable_field_effect(card, true);
            self.adjust_instant();
        }
        if proc {
            self.cards[card].set_status(STATUS_PROC_COMPLETE, true);
        }
    }

    // interpreter::register_card: the card's script (its alias's for alternate arts) registers its initial effects.
    pub(crate) fn register_card(&mut self, card: CardId) {
        let data = self.cards[card].data;
        let Some(library) = self.library else { return };
        let code = if data.alias != 0 && data.alias < data.code + 10 && data.code < data.alias + 10 {
            data.alias
        } else {
            data.code
        };
        if data.code == 0 {
            return;
        }
        if let Some(initial_effect) = (library.card_script)(code) {
            self.cards[card].set_status(STATUS_INITIALIZING, true);
            initial_effect(self, card);
            self.cards[card].set_status(STATUS_INITIALIZING, false);
        }
    }

    pub fn add_deck(&mut self, team: u8, deck: &Deck, database: &'static CardDatabase) -> Result<(), DuelError> {
        let main = deck.main.iter().map(|&code| (code, LOCATION_DECK));
        for (code, location) in main.chain(deck.extra.iter().map(|&code| (code, LOCATION_EXTRA))) {
            let info = NewCardInfo {
                team,
                duelist: 0,
                code,
                con: team,
                loc: u32::from(location),
                seq: 0,
                pos: POS_FACEDOWN_DEFENSE,
            };
            self.new_card(info, database)?;
        }
        Ok(())
    }

    pub fn start(&mut self) {
        self.emplace_process(Startup::default());
    }

    // Runs until a prompt or MSG_WIN; unlike the core, which keeps going after MSG_WIN, the duel then stays ended.
    // A pending prompt is left waiting for `apply`.
    pub fn process(&mut self) -> Status {
        if self.prompt().is_some() {
            return Status::Awaiting;
        }
        self.run()
    }

    pub(crate) fn run(&mut self) -> Status {
        // While units run, only emplace_process touches the unit stacks, and it pushes to core.subunits.
        let mut units = std::mem::take(&mut self.core.units);
        let status = loop {
            if self.result.is_some() {
                break Status::End;
            }
            match self.process_unit(&mut units) {
                OcgStatus::Awaiting => break Status::Awaiting,
                OcgStatus::End => break Status::End,
                OcgStatus::Continue => {}
            }
        };
        self.core.units = units;
        status
    }

    pub fn phase(&self) -> u16 {
        self.infos.phase
    }

    // The winner (PLAYER_NONE for a draw) and the reason of MSG_WIN.
    pub fn result(&self) -> Option<(u8, u8)> {
        self.result
    }

    // The first call site an effect reached whose handling is not ported yet ("file:line"); once set, the duel may
    // have diverged from the core, so its result cannot be trusted.
    pub fn unported_site(&self) -> Option<&'static str> {
        self.unported.get()
    }

    pub fn reach_unported(&self, site: &'static str) {
        debug_assert!(false, "an effect reached the unported call site {site}");
        if self.unported.get().is_none() {
            self.unported.set(Some(site));
        }
    }
}
