// Fixed-size integer tensors of an observation, column names in order. Players are relative to the observer (0 itself,
// 1 its opponent, 2 neither, as the winner of a draw), -1 is none, and codes are the raw u32 bits.
use engine::{Action, Observation, Prompt, SeenCard};
use rayon::prelude::*;

use crate::encoding::{ACTIONS, EncodingError, indices, slots};

pub const GLOBAL_FEATURES: [&str; 10] = [
    "turn",
    "turn_player",
    "phase",
    "lp",
    "opponent_lp",
    "acting",
    "prompt",
    "chain",
    "winner",
    "reason",
];
// In the order of Observation::cards; a row past them is all 0.
pub const CARD_FEATURES: [&str; 8] = [
    "valid",
    "controller",
    "location",
    "sequence",
    "position",
    "code",
    "attack",
    "defense",
];
// A description aux.Stringid(code, index) is code << 4 | index.
pub const CHAIN_FEATURES: [&str; 8] = [
    "valid",
    "controller",
    "code",
    "location",
    "sequence",
    "description_code",
    "description_index",
    "targets",
];
// A row per action index: the section, the card table row of the card picked, that card's code as the player picking
// sees it or the code announced, and the description; -1 for section and row if not legal.
pub const ACTION_FEATURES: [&str; 5] = ["section", "row", "code", "description_code", "description_index"];
pub const PROMPTS: [&str; 16] = [
    "IdleCommand",
    "BattleCommand",
    "YesNo",
    "Option",
    "EffectYesNo",
    "Chain",
    "Card",
    "Tribute",
    "Sum",
    "UnselectCard",
    "Place",
    "Position",
    "AnnounceRace",
    "AnnounceAttribute",
    "Sort",
    "AnnounceCard",
];
// Legal decks put at most 150 cards in a duel, then tokens; see PLAN.md, 5d findings, for the counts measured.
pub const MAX_OBS_CARDS: usize = 192;
pub const MAX_CHAIN: usize = 16;

type GlobalRow = [i32; GLOBAL_FEATURES.len()];
type CardRow = [i32; CARD_FEATURES.len()];
type ChainRow = [i32; CHAIN_FEATURES.len()];
type ActionRow = [i32; ACTION_FEATURES.len()];

const NO_ACTION: ActionRow = [-1, -1, 0, 0, 0];

// Buffers for a number of observations, each in the shapes [GLOBAL_FEATURES], [MAX_OBS_CARDS, CARD_FEATURES],
// [MAX_CHAIN, CHAIN_FEATURES] and [ACTIONS, ACTION_FEATURES].
pub struct Tensors<'a> {
    global: &'a mut [GlobalRow],
    cards: &'a mut [[CardRow; MAX_OBS_CARDS]],
    chain: &'a mut [[ChainRow; MAX_CHAIN]],
    actions: &'a mut [[ActionRow; ACTIONS]],
}

struct Row<'a> {
    global: &'a mut GlobalRow,
    cards: &'a mut [CardRow; MAX_OBS_CARDS],
    chain: &'a mut [ChainRow; MAX_CHAIN],
    actions: &'a mut [ActionRow; ACTIONS],
}

fn tables<const COLUMNS: usize, const ROWS: usize>(flat: &mut [i32]) -> Option<&mut [[[i32; COLUMNS]; ROWS]]> {
    let (rows, []) = flat.as_chunks_mut::<COLUMNS>() else { return None };
    let (tables, []) = rows.as_chunks_mut::<ROWS>() else { return None };
    Some(tables)
}

impl<'a> Tensors<'a> {
    pub fn new(global: &'a mut [i32], cards: &'a mut [i32], chain: &'a mut [i32], actions: &'a mut [i32]) -> Result<Self, EncodingError> {
        let tensors = Self::split(global, cards, chain, actions);
        let count = |tensors: &Self| [tensors.cards.len(), tensors.chain.len(), tensors.actions.len()];
        tensors
            .filter(|tensors| count(tensors) == [tensors.len(); 3])
            .ok_or(EncodingError::Buffers)
    }

    fn split(global: &'a mut [i32], cards: &'a mut [i32], chain: &'a mut [i32], actions: &'a mut [i32]) -> Option<Self> {
        let (global, []) = global.as_chunks_mut() else { return None };
        Some(Self {
            global,
            cards: tables(cards)?,
            chain: tables(chain)?,
            actions: tables(actions)?,
        })
    }

    pub fn len(&self) -> usize {
        self.global.len()
    }

    pub fn is_empty(&self) -> bool {
        self.global.is_empty()
    }

    // Writes observation number `index`, or all empty rows for None.
    pub fn write(&mut self, index: usize, observation: Option<&Observation>) -> Result<(), EncodingError> {
        let rows = self.len();
        self.row(index).ok_or(EncodingError::Row { index, rows })?.write(observation)
    }

    fn row(&mut self, index: usize) -> Option<Row<'_>> {
        Some(Row {
            global: self.global.get_mut(index)?,
            cards: self.cards.get_mut(index)?,
            chain: self.chain.get_mut(index)?,
            actions: self.actions.get_mut(index)?,
        })
    }

    pub(crate) fn par_write<T: Send>(
        &mut self,
        items: impl IndexedParallelIterator<Item = T>,
        observe: impl Fn(T) -> Option<Observation> + Sync,
    ) -> Result<(), (usize, EncodingError)> {
        let rows = self.global.par_iter_mut().zip(self.cards.par_iter_mut());
        let rows = rows.zip(self.chain.par_iter_mut()).zip(self.actions.par_iter_mut());
        let rows = rows.map(|(((global, cards), chain), actions)| Row {
            global,
            cards,
            chain,
            actions,
        });
        rows.zip(items)
            .enumerate()
            .try_for_each(|(index, (row, item))| row.write(observe(item).as_ref()).map_err(|error| (index, error)))
    }
}

fn prompt_kind(prompt: Prompt) -> i32 {
    match prompt {
        Prompt::IdleCommand { .. } => 0,
        Prompt::BattleCommand { .. } => 1,
        Prompt::YesNo { .. } => 2,
        Prompt::Option { .. } => 3,
        Prompt::EffectYesNo { .. } => 4,
        Prompt::Chain { .. } => 5,
        Prompt::Card { .. } => 6,
        Prompt::Tribute { .. } => 7,
        Prompt::Sum { .. } => 8,
        Prompt::UnselectCard { .. } => 9,
        Prompt::Place { .. } => 10,
        Prompt::Position { .. } => 11,
        Prompt::AnnounceRace { .. } => 12,
        Prompt::AnnounceAttribute { .. } => 13,
        Prompt::Sort { .. } => 14,
        Prompt::AnnounceCard { .. } => 15,
    }
}

fn place(card: &SeenCard) -> [u8; 4] {
    [card.controller, card.location, card.sequence, card.position]
}

fn split(description: u64) -> [i32; 2] {
    [(description >> 4) as i32, (description & 0xf) as i32]
}

impl Row<'_> {
    fn write(self, observation: Option<&Observation>) -> Result<(), EncodingError> {
        let Self {
            global,
            cards,
            chain,
            actions,
        } = self;
        *global = [0; GLOBAL_FEATURES.len()];
        cards.fill([0; CARD_FEATURES.len()]);
        chain.fill([0; CHAIN_FEATURES.len()]);
        actions.fill(NO_ACTION);
        let Some(observation) = observation else { return Ok(()) };
        let seen = &observation.cards;
        if seen.len() > MAX_OBS_CARDS {
            return Err(EncodingError::Cards(seen.len()));
        }
        if observation.chain.len() > MAX_CHAIN {
            return Err(EncodingError::Chain(observation.chain.len()));
        }
        let relative = |player: u8| i32::from(if player < 2 { player ^ observation.player } else { player });
        for (row, card) in cards.iter_mut().zip(seen) {
            let SeenCard {
                controller,
                location,
                sequence,
                position,
                code,
                attack,
                defense,
            } = *card;
            let [location, sequence, position] = [location, sequence, position].map(i32::from);
            *row = [1, relative(controller), location, sequence, position, code as i32, attack, defense];
        }
        for (row, link) in chain.iter_mut().zip(&observation.chain) {
            let [description_code, description_index] = split(link.description);
            *row = [
                1,
                relative(link.controller),
                link.code as i32,
                link.location.into(),
                link.sequence as i32,
                description_code,
                description_index,
                link.targets.len() as i32,
            ];
        }
        let choices = &observation.choices;
        let legal = || choices.iter().map(|choice| &choice.action);
        for ((choice, (section, _)), index) in choices.iter().zip(slots(legal())).zip(indices(legal())) {
            let card_row = choice
                .card
                .and_then(|card| seen.iter().position(|other| place(other) == place(&card)));
            let code = match (choice.action, choice.card) {
                (Action::Code(code), _) | (_, Some(SeenCard { code, .. })) => code as i32,
                _ => 0,
            };
            let [description_code, description_index] = split(choice.description);
            if let Some(row) = actions.get_mut(index?) {
                *row = [
                    section as i32,
                    card_row.map_or(-1, |row| row as i32),
                    code,
                    description_code,
                    description_index,
                ];
            }
        }
        let [lp0, lp1] = observation.lp;
        let (lp, opponent_lp) = if observation.player == 0 { (lp0, lp1) } else { (lp1, lp0) };
        let result = observation.result;
        *global = [
            observation.turn.into(),
            relative(observation.turn_player),
            observation.phase.into(),
            lp,
            opponent_lp,
            observation.acting.map_or(-1, relative),
            observation.prompt.map_or(-1, prompt_kind),
            observation.chain.len() as i32,
            result.map_or(-1, |(winner, _)| relative(winner)),
            result.map_or(-1, |(_, reason)| reason.into()),
        ];
        Ok(())
    }
}
