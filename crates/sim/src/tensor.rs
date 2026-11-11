// Players are relative to the observer (0 itself, 1 its opponent, 2 neither), -1 is none, codes are the raw u32 bits.
use engine::{Action, Choice, Duel, Observation, ObservationSink, Prompt, SeenCard, SeenLink};
use rayon::prelude::*;

use crate::encoding::{ACTIONS, EncodingError, slot};

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
// A row per action index, the code as the picking player sees it or as announced; -1 for section and row if not legal.
pub const ACTION_FEATURES: [&str; 5] = ["section", "row", "code", "description_code", "description_index"];
pub const PROMPTS: [&str; 17] = [
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
    "Counter",
];
// Legal decks put at most 150 cards in a duel, then tokens; see PLAN.md, 5d findings, for the counts measured.
pub const MAX_OBS_CARDS: usize = 192;
pub const MAX_CHAIN: usize = 16;

type GlobalRow = [i32; GLOBAL_FEATURES.len()];
type CardRow = [i32; CARD_FEATURES.len()];
type ChainRow = [i32; CHAIN_FEATURES.len()];
type ActionRow = [i32; ACTION_FEATURES.len()];

const NO_ACTION: ActionRow = [-1, -1, 0, 0, 0];

pub const SHAPES: [&[usize]; 4] = [
    &[GLOBAL_FEATURES.len()],
    &[MAX_OBS_CARDS, CARD_FEATURES.len()],
    &[MAX_CHAIN, CHAIN_FEATURES.len()],
    &[ACTIONS, ACTION_FEATURES.len()],
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffers(pub [Vec<i32>; 4]);

impl Buffers {
    pub fn new(rows: usize) -> Self {
        Self(SHAPES.map(|shape| vec![0; rows * shape.iter().product::<usize>()]))
    }

    pub fn tensors(&mut self) -> Result<Tensors<'_>, EncodingError> {
        let [global, cards, chain, actions] = &mut self.0;
        Tensors::new(global, cards, chain, actions)
    }
}

// Buffers for a number of observations, each in the shapes of [SHAPES].
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

    // As `write(index, Some(&duel.observe(player)))`, without building the observation.
    pub fn write_duel(&mut self, index: usize, duel: &Duel, player: u8) -> Result<(), EncodingError> {
        let rows = self.len();
        self.row(index).ok_or(EncodingError::Row { index, rows })?.write_duel(duel, player)
    }

    fn row(&mut self, index: usize) -> Option<Row<'_>> {
        Some(Row {
            global: self.global.get_mut(index)?,
            cards: self.cards.get_mut(index)?,
            chain: self.chain.get_mut(index)?,
            actions: self.actions.get_mut(index)?,
        })
    }

    // Each duel's observation for the player its prompt waits for, empty rows for None or no prompt.
    pub(crate) fn par_write<'d>(
        &mut self,
        duels: impl IndexedParallelIterator<Item = Option<&'d mut Duel>>,
    ) -> Result<(), (usize, EncodingError)> {
        let rows = self.global.par_iter_mut().zip(self.cards.par_iter_mut());
        let rows = rows.zip(self.chain.par_iter_mut()).zip(self.actions.par_iter_mut());
        let rows = rows.map(|(((global, cards), chain), actions)| Row {
            global,
            cards,
            chain,
            actions,
        });
        rows.zip(duels).enumerate().try_for_each(|(index, (row, duel))| {
            let prompted = duel.and_then(|duel| Some((duel.prompt()?.player(), duel)));
            let written = match prompted {
                Some((player, duel)) => row.write_duel(duel, player),
                None => row.write(None),
            };
            written.map_err(|error| (index, error))
        })
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
        Prompt::Counter { .. } => 16,
    }
}

fn split(description: u64) -> [i32; 2] {
    [(description >> 4) as i32, (description & 0xf) as i32]
}

impl Row<'_> {
    fn write(self, observation: Option<&Observation>) -> Result<(), EncodingError> {
        let mut writer = Writer::new(self, observation.map_or(0, |observation| observation.player));
        let Some(observation) = observation else { return Ok(()) };
        observation.cards.iter().for_each(|&card| writer.card(card));
        for link in &observation.chain {
            writer.link(link);
            link.targets.iter().for_each(|&card| writer.target(card));
        }
        observation.choices.iter().for_each(|&choice| writer.choice(choice));
        writer.finish(observation)
    }

    fn write_duel(self, duel: &Duel, player: u8) -> Result<(), EncodingError> {
        let mut writer = Writer::new(self, player);
        let observation = duel.observe_into(player, &mut writer);
        writer.finish(&observation)
    }
}

// Encodes an observation as it streams in, counting what goes past the tables' rows.
struct Writer<'a> {
    row: Row<'a>,
    player: u8,
    cards: usize,
    links: usize,
    codes: usize,
    overflow: Option<EncodingError>,
}

impl<'a> Writer<'a> {
    fn new(row: Row<'a>, player: u8) -> Self {
        *row.global = [0; GLOBAL_FEATURES.len()];
        row.cards.fill([0; CARD_FEATURES.len()]);
        row.chain.fill([0; CHAIN_FEATURES.len()]);
        row.actions.fill(NO_ACTION);
        Self {
            row,
            player,
            cards: 0,
            links: 0,
            codes: 0,
            overflow: None,
        }
    }

    fn relative(&self, player: u8) -> i32 {
        i32::from(if player < 2 { player ^ self.player } else { player })
    }

    fn place(&self, card: &SeenCard) -> [i32; 4] {
        let [location, sequence, position] = [card.location, card.sequence, card.position].map(i32::from);
        [self.relative(card.controller), location, sequence, position]
    }

    fn finish(self, observation: &Observation) -> Result<(), EncodingError> {
        if self.cards > MAX_OBS_CARDS {
            return Err(EncodingError::Cards(self.cards));
        }
        if self.links > MAX_CHAIN {
            return Err(EncodingError::Chain(self.links));
        }
        if let Some(error) = self.overflow {
            return Err(error);
        }
        let [lp0, lp1] = observation.lp;
        let (lp, opponent_lp) = if observation.player == 0 { (lp0, lp1) } else { (lp1, lp0) };
        let result = observation.result;
        *self.row.global = [
            observation.turn.into(),
            self.relative(observation.turn_player),
            observation.phase.into(),
            lp,
            opponent_lp,
            observation.acting.map_or(-1, |player| self.relative(player)),
            observation.prompt.map_or(-1, prompt_kind),
            self.links as i32,
            result.map_or(-1, |(winner, _)| self.relative(winner)),
            result.map_or(-1, |(_, reason)| reason.into()),
        ];
        Ok(())
    }
}

impl ObservationSink for Writer<'_> {
    fn card(&mut self, card: SeenCard) {
        let [controller, location, sequence, position] = self.place(&card);
        if let Some(row) = self.row.cards.get_mut(self.cards) {
            *row = [
                1,
                controller,
                location,
                sequence,
                position,
                card.code as i32,
                card.attack,
                card.defense,
            ];
        }
        self.cards += 1;
    }

    fn link(&mut self, link: &SeenLink) {
        let [description_code, description_index] = split(link.description);
        let controller = self.relative(link.controller);
        if let Some(row) = self.row.chain.get_mut(self.links) {
            *row = [
                1,
                controller,
                link.code as i32,
                link.location.into(),
                link.sequence as i32,
                description_code,
                description_index,
                0,
            ];
        }
        self.links += 1;
    }

    fn target(&mut self, _: SeenCard) {
        let row = self.links.checked_sub(1).and_then(|link| self.row.chain.get_mut(link));
        if let Some(targets) = row.and_then(|row| row.last_mut()) {
            *targets += 1;
        }
    }

    fn choice(&mut self, choice: Choice) {
        let (section, slot) = slot(choice.action, self.codes);
        self.codes += usize::from(matches!(choice.action, Action::Code(_)));
        if slot >= section.slots() {
            self.overflow.get_or_insert(EncodingError::Overflow { section, slot });
            return;
        }
        let place = choice.card.map(|card| self.place(&card));
        let mut seen = self.row.cards.iter().take(self.cards);
        let card_row = place.and_then(|place| seen.position(|&[_, c, l, s, p, ..]| [c, l, s, p] == place));
        let code = match (choice.action, choice.card) {
            (Action::Code(code), _) | (_, Some(SeenCard { code, .. })) => code as i32,
            _ => 0,
        };
        let [description_code, description_index] = split(choice.description);
        if let Some(row) = self.row.actions.get_mut(section.start() + slot) {
            *row = [
                section as i32,
                card_row.map_or(-1, |row| row as i32),
                code,
                description_code,
                description_index,
            ];
        }
    }
}
