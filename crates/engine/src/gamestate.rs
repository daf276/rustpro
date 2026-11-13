// Gamestates outside the process; effects hold function pointers, so a Position describes a duel and a DuelRecord replays one.
use carddata::{CardDatabase, Deck};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::action::{Action, ActionError};
use crate::card::CardId;
use crate::constants::*;
use crate::duel::{Duel, DuelError, DuelOptions, NewCardInfo, PlayerOptions, deck_cards};
use crate::processor::{Startup, Turn};
use crate::script::Library;

// A puzzle's setup, not a snapshot: lingering effects, relations, summon types and a chain or prompt in progress are left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub flags: u64,
    pub players: [PlayerOptions; 2],
    pub turn: u16,
    pub turn_player: u8,
    pub phase: u16,
    pub cards: Vec<PlacedCard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedCard {
    pub info: NewCardInfo,
    pub proc_complete: bool,
    // The counter types of card::counters with their permanent count and the count a negation removes.
    pub counters: Vec<(u16, [u16; 2])>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PositionError {
    #[error(transparent)]
    Duel(#[from] DuelError),
    #[error("a duel cannot start in turn {turn} of player {player}")]
    Turn { turn: u16, player: u8 },
    #[error("a duel cannot start in phase {0:#x}")]
    Phase(u16),
    #[error("{0:?} is not a place to put a card")]
    Place(NewCardInfo),
}

impl Position {
    // Turn 1 of player 0 from its Draw Phase, without cards.
    pub fn new(flags: u64, players: [PlayerOptions; 2]) -> Self {
        Self {
            flags,
            players,
            turn: 1,
            turn_player: 0,
            phase: PHASE_DRAW,
            cards: Vec::new(),
        }
    }

    pub fn add_deck(&mut self, team: u8, deck: &Deck) {
        self.cards.extend(deck_cards(team, deck).map(|info| PlacedCard {
            info,
            proc_complete: false,
            counters: Vec::new(),
        }));
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, FormatError> {
        encode(*b"YGSP", self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FormatError> {
        decode(*b"YGSP", bytes)
    }
}

// The step of processor::Turn that begins a phase, and the phase a position names it by: the Battle Phase starts over.
fn turn_start(phase: u16) -> Option<(u16, u16)> {
    Some(match phase {
        PHASE_DRAW => (0, PHASE_DRAW),
        PHASE_STANDBY => (5, PHASE_STANDBY),
        PHASE_MAIN1 => (7, PHASE_MAIN1),
        PHASE_BATTLE_START | PHASE_BATTLE_STEP | PHASE_DAMAGE | PHASE_DAMAGE_CAL | PHASE_BATTLE => (10, PHASE_BATTLE_START),
        PHASE_MAIN2 => (13, PHASE_MAIN2),
        PHASE_END => (16, PHASE_END),
        _ => return None,
    })
}

// The places Debug.AddCard accepts that the placement handles, by location and number of sequences.
fn is_place(info: &NewCardInfo) -> bool {
    let places: [(u32, u32); 9] = [
        (LOCATION_DECK.into(), 2),
        (LOCATION_HAND.into(), 1),
        (LOCATION_MZONE.into(), 7),
        (LOCATION_SZONE.into(), 8),
        (LOCATION_GRAVE.into(), 1),
        (LOCATION_REMOVED.into(), 1),
        (LOCATION_EXTRA.into(), 1),
        (LOCATION_PZONE.into(), 2),
        (LOCATION_FZONE.into(), 1),
    ];
    let players = info.team <= 1 && info.con <= 1 && info.duelist == 0;
    players && places.iter().any(|&(loc, count)| loc == info.loc && info.seq < count)
}

impl Duel {
    // A duel set up at `position`, ready to `process`: its cards' scripts register their effects as they are placed, and at
    // the start each player draws its `starting_draw_count`, as a duel from its decks does (a record's position relies on it).
    pub fn from_position(
        position: &Position,
        database: &'static CardDatabase,
        library: &'static Library,
        seed: [u64; 4],
    ) -> Result<Self, PositionError> {
        let (turn, player) = (position.turn, position.turn_player);
        if turn == 0 || turn > i16::MAX as u16 || player > 1 {
            return Err(PositionError::Turn { turn, player });
        }
        let (step, phase) = turn_start(position.phase).ok_or(PositionError::Phase(position.phase))?;
        let [team1, team2] = position.players;
        let mut duel = Self::new(DuelOptions {
            seed,
            flags: position.flags,
            team1,
            team2,
        })?;
        duel.load_library(library);
        for card in &position.cards {
            if !is_place(&card.info) {
                return Err(PositionError::Place(card.info));
            }
            let Some(id) = duel.debug_add_card(card.info, card.proc_complete, database)? else {
                continue;
            };
            for &(counter_type, [permanent, negatable]) in &card.counters {
                duel.debug_pre_add_counter(id, counter_type, permanent);
                duel.debug_pre_add_counter(id, counter_type | COUNTER_NEED_ENABLE, negatable);
            }
        }
        // The turns begun before, counted alternately; a duel starting at a Draw Phase begins that turn itself.
        let begun = turn - u16::from(step == 0);
        let first = if turn % 2 == 1 { player } else { 1 - player };
        let infos = &mut duel.infos;
        infos.turn_id = begun as i16;
        infos.turn_id_by_player[first] = begun.div_ceil(2) as i16;
        infos.turn_id_by_player[1 - first] = (begun / 2) as i16;
        infos.turn_player = player;
        infos.phase = if step == 0 { 0 } else { phase };
        duel.emplace_process(Startup {
            turn: Turn {
                step,
                turn_player: player,
                ..Turn::default()
            },
            ..Startup::default()
        });
        Ok(duel)
    }

    // What a puzzle sets up to start from this duel's cards, LP and turn (see [Position]), with a `starting_draw_count` of 0
    // so that the hands stay as they are.
    pub fn position(&self) -> Position {
        let infos = &self.infos;
        let (turn, phase) = match infos.phase {
            0 => (infos.turn_id + 1, PHASE_DRAW),
            phase => (infos.turn_id, turn_start(phase).map_or(phase, |(_, start)| start)),
        };
        let lp = |p: u8| PlayerOptions {
            starting_lp: self.player[p].lp as u32,
            starting_draw_count: 0,
            draw_count_per_turn: self.player[p].draw_count as u32,
        };
        Position {
            flags: self.core.duel_options,
            players: [lp(0), lp(1)],
            turn: turn as u16,
            turn_player: infos.turn_player,
            phase,
            cards: self.located_cards().into_iter().map(|card| self.placed_card(card)).collect(),
        }
    }

    fn placed_card(&self, card: CardId) -> PlacedCard {
        let pcard = &self.cards[card];
        let (current, pos) = match pcard.overlay_target {
            Some(xyz) => (self.cards[xyz].current, 0),
            // Debug.AddCard puts it face-down in Defense Position, so a position cannot carry Duel.Sendto's POS_FACEDOWN there.
            None if pcard.current.location == LOCATION_EXTRA && pcard.data.card_type & TYPE_PENDULUM == 0 => {
                (pcard.current, POS_FACEDOWN_DEFENSE)
            }
            None => (pcard.current, pcard.current.position),
        };
        let (loc, seq) = if current.location & LOCATION_ONFIELD == 0 {
            (current.location.into(), 0)
        } else if current.pzone && pcard.overlay_target.is_none() {
            (LOCATION_PZONE.into(), u32::from(self.get_pzone_index(0) != current.sequence))
        } else {
            (current.location.into(), current.sequence)
        };
        PlacedCard {
            info: NewCardInfo {
                team: pcard.owner,
                duelist: 0,
                code: pcard.data.code,
                con: current.controler,
                loc,
                seq,
                pos,
            },
            proc_complete: pcard.is_status(STATUS_PROC_COMPLETE),
            counters: (self.counters.iter())
                .filter(|&&(other, ..)| other == card)
                .map(|&(_, counter_type, counts)| (counter_type, counts))
                .collect(),
        }
    }
}

// The setup, the seed and every action since, which is all a duel depends on, so a replay reaches the recorded state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuelRecord {
    pub seed: [u64; 4],
    pub position: Position,
    pub actions: Vec<Action>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplayError {
    #[error(transparent)]
    Position(#[from] PositionError),
    #[error("action {index}: {error}")]
    Action { index: usize, error: ActionError },
}

impl DuelRecord {
    pub fn replay(&self, database: &'static CardDatabase, library: &'static Library) -> Result<Duel, ReplayError> {
        let mut duel = Duel::from_position(&self.position, database, library, self.seed)?;
        duel.process();
        for (index, &action) in self.actions.iter().enumerate() {
            duel.apply(action).map_err(|error| ReplayError::Action { index, error })?;
        }
        Ok(duel)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, FormatError> {
        encode(*b"YGSR", self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FormatError> {
        decode(*b"YGSR", bytes)
    }
}

// Both formats start with a tag naming the type and the format version, which any change to the types must bump.
const VERSION: u16 = 1;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FormatError {
    #[error("not data of this type")]
    Tag,
    #[error("format version {0}, this build reads {VERSION}")]
    Version(u16),
    #[error("malformed data: {0}")]
    Data(postcard::Error),
}

fn encode<T: Serialize>(tag: [u8; 4], value: &T) -> Result<Vec<u8>, FormatError> {
    let mut bytes = [&tag[..], &VERSION.to_le_bytes()].concat();
    bytes.extend(postcard::to_allocvec(value).map_err(FormatError::Data)?);
    Ok(bytes)
}

fn decode<T: DeserializeOwned>(tag: [u8; 4], bytes: &[u8]) -> Result<T, FormatError> {
    let (&[t0, t1, t2, t3, v0, v1], data) = bytes.split_first_chunk::<6>().ok_or(FormatError::Tag)?;
    if [t0, t1, t2, t3] != tag {
        return Err(FormatError::Tag);
    }
    let version = u16::from_le_bytes([v0, v1]);
    if version != VERSION {
        return Err(FormatError::Version(version));
    }
    postcard::from_bytes(data).map_err(FormatError::Data)
}
