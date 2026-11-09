// One section per kind of action, a slot per entry of the prompt's list it indexes (what entry i is comes from the
// observation's choices) or per zone, position, race or attribute bit; prompts indexing the same list share a section.
use engine::{Action, BattleCommand, Duel, IdleCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    // Pass, ToBattle, ToMain2, ToEnd, Shuffle, Finish, Cancel, No, Yes.
    Command,
    Card,
    // IdleCommand::Activate, BattleCommand::Activate and Chain(Some).
    Activate,
    Summon,
    SpecialSummon,
    Reposition,
    MonsterSet,
    SpellSet,
    Attack,
    Option,
    // The code's place among the codes [Duel::policy_actions] offers.
    Code,
    Place,
    Position,
    Race,
    Attribute,
}

impl Section {
    pub const ALL: [Self; 15] = [
        Self::Command,
        Self::Card,
        Self::Activate,
        Self::Summon,
        Self::SpecialSummon,
        Self::Reposition,
        Self::MonsterSet,
        Self::SpellSet,
        Self::Attack,
        Self::Option,
        Self::Code,
        Self::Place,
        Self::Position,
        Self::Race,
        Self::Attribute,
    ];

    // List-indexed sections hold about twice the longest list measured (PLAN.md, 5c findings); Reposition and Attack
    // all seven Monster Zones.
    pub const fn slots(self) -> usize {
        match self {
            Self::Command => 9,
            Self::Card => 160,
            Self::Activate => 16,
            Self::Summon | Self::SpecialSummon | Self::MonsterSet => 20,
            Self::Reposition | Self::Attack | Self::Option => 8,
            Self::SpellSet => 16,
            Self::Code => engine::MAX_DECLARED_CODES,
            Self::Place => 32,
            Self::Position => 4,
            Self::Race => 64,
            Self::Attribute => 7,
        }
    }

    pub const fn start(self) -> usize {
        total(Self::ALL.split_at(self as usize).0)
    }
}

const fn total(mut sections: &[Section]) -> usize {
    let mut total = 0;
    while let [section, rest @ ..] = sections {
        total += section.slots();
        sections = rest;
    }
    total
}

pub const ACTIONS: usize = total(&Section::ALL);

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum EncodingError {
    #[error("slot {slot} is beyond the {} slots of section {section:?}", section.slots())]
    Overflow { section: Section, slot: usize },
    #[error("{0} cards are beyond the {max} rows of the card table", max = crate::MAX_OBS_CARDS)]
    Cards(usize),
    #[error("a chain of {0} links is beyond the {max} rows of the chain table", max = crate::MAX_CHAIN)]
    Chain(usize),
    #[error("the buffers do not hold the same whole number of observations")]
    Buffers,
    #[error("observation {index} of buffers for {rows}")]
    Row { index: usize, rows: usize },
}

pub(crate) fn slot(action: Action, code: usize) -> (Section, usize) {
    let command = |slot| (Section::Command, slot);
    match action {
        Action::Chain(None) => command(0),
        Action::IdleCommand(IdleCommand::ToBattle) => command(1),
        Action::BattleCommand(BattleCommand::ToMain2) => command(2),
        Action::IdleCommand(IdleCommand::ToEnd) | Action::BattleCommand(BattleCommand::ToEnd) => command(3),
        Action::IdleCommand(IdleCommand::Shuffle) => command(4),
        Action::Finish => command(5),
        Action::Cancel => command(6),
        Action::YesNo(yes) => command(7 + usize::from(yes)),
        Action::Card(index) => (Section::Card, index.into()),
        Action::IdleCommand(IdleCommand::Activate(index))
        | Action::BattleCommand(BattleCommand::Activate(index))
        | Action::Chain(Some(index)) => (Section::Activate, index.into()),
        Action::IdleCommand(IdleCommand::Summon(index)) => (Section::Summon, index.into()),
        Action::IdleCommand(IdleCommand::SpecialSummon(index)) => (Section::SpecialSummon, index.into()),
        Action::IdleCommand(IdleCommand::Reposition(index)) => (Section::Reposition, index.into()),
        Action::IdleCommand(IdleCommand::MonsterSet(index)) => (Section::MonsterSet, index.into()),
        Action::IdleCommand(IdleCommand::SpellSet(index)) => (Section::SpellSet, index.into()),
        Action::BattleCommand(BattleCommand::Attack(index)) => (Section::Attack, index.into()),
        Action::Option(index) => (Section::Option, index.into()),
        Action::Code(_) => (Section::Code, code),
        Action::Place(zone) => (Section::Place, zone.into()),
        Action::Position(position) => (Section::Position, position.trailing_zeros() as usize),
        Action::Race(bit) => (Section::Race, bit.into()),
        Action::Attribute(bit) => (Section::Attribute, bit.into()),
    }
}

// Uncapped, so that list lengths can be measured.
pub fn slots<'a>(legal: impl IntoIterator<Item = &'a Action>) -> impl Iterator<Item = (Section, usize)> {
    legal.into_iter().scan(0, |codes, &action| {
        let code = *codes;
        *codes += usize::from(matches!(action, Action::Code(_)));
        Some(slot(action, code))
    })
}

// In the order of the legal actions given, so a feature row per entry of Observation::choices lines up with the mask.
pub fn indices<'a>(legal: impl IntoIterator<Item = &'a Action>) -> impl Iterator<Item = Result<usize, EncodingError>> {
    slots(legal).map(|(section, slot)| match slot < section.slots() {
        true => Ok(section.start() + slot),
        false => Err(EncodingError::Overflow { section, slot }),
    })
}

pub fn write_mask(legal: &[Action], mask: &mut [bool]) -> Result<(), EncodingError> {
    mask.fill(false);
    for index in indices(legal) {
        if let Some(legal) = mask.get_mut(index?) {
            *legal = true;
        }
    }
    Ok(())
}

pub fn find(legal: &[Action], index: usize) -> Option<Action> {
    legal
        .iter()
        .zip(indices(legal))
        .find(|(_, other)| *other == Ok(index))
        .map(|(&action, _)| action)
}

// None also for an action beyond its section's slots.
pub fn encode(duel: &Duel, action: Action) -> Option<usize> {
    let legal = duel.policy_actions();
    legal.iter().zip(indices(&legal)).find(|&(&other, _)| other == action)?.1.ok()
}

pub fn decode(duel: &Duel, index: usize) -> Option<Action> {
    find(&duel.policy_actions(), index)
}
