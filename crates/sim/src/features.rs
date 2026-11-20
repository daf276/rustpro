// The sparse input of a value network, NNUE-style: a bag of feature indices summed into its first layer, so deck and hand sizes
// vary freely. Copies repeat an index, as a sum of the same row is a count; a copy index per card would triple the card table.
use engine::constants::*;
use engine::{Duel, SeenCard};

use crate::tracker::Tracker;

// Card rows before the vocabulary's: a card whose code the player does not know, and a known card outside the vocabulary.
pub const HIDDEN: usize = 0;
pub const OTHER: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    Deck,
    Hand,
    MonsterAttack,
    MonsterDefense,
    MonsterSet,
    SpellTrap,
    SpellTrapSet,
    FieldSpell,
    FieldSpellSet,
    Graveyard,
    Banished,
    BanishedSet,
    ExtraDeck,
    ExtraDeckFaceUp,
    Material,
}

impl Place {
    pub const ALL: [Self; 15] = [
        Self::Deck,
        Self::Hand,
        Self::MonsterAttack,
        Self::MonsterDefense,
        Self::MonsterSet,
        Self::SpellTrap,
        Self::SpellTrapSet,
        Self::FieldSpell,
        Self::FieldSpellSet,
        Self::Graveyard,
        Self::Banished,
        Self::BanishedSet,
        Self::ExtraDeck,
        Self::ExtraDeckFaceUp,
        Self::Material,
    ];

    fn of(card: &SeenCard) -> Self {
        let position = u32::from(card.position);
        let face_down = position & POS_FACEDOWN != 0;
        match card.location {
            location if location & LOCATION_OVERLAY != 0 => Self::Material,
            LOCATION_DECK => Self::Deck,
            LOCATION_HAND => Self::Hand,
            LOCATION_MZONE if face_down => Self::MonsterSet,
            LOCATION_MZONE if position & POS_DEFENSE != 0 => Self::MonsterDefense,
            LOCATION_MZONE => Self::MonsterAttack,
            LOCATION_SZONE if card.sequence == 5 && face_down => Self::FieldSpellSet,
            LOCATION_SZONE if card.sequence == 5 => Self::FieldSpell,
            LOCATION_SZONE if face_down => Self::SpellTrapSet,
            LOCATION_SZONE => Self::SpellTrap,
            LOCATION_GRAVE => Self::Graveyard,
            LOCATION_REMOVED if face_down => Self::BanishedSet,
            LOCATION_REMOVED => Self::Banished,
            _ if face_down => Self::ExtraDeck,
            _ => Self::ExtraDeckFaceUp,
        }
    }
}

// Bucket edges: a value falls in the bucket of the edges at or below it.
const HAND_SIZES: [i64; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const DECK_SIZES: [i64; 11] = [1, 2, 3, 4, 5, 10, 15, 20, 25, 30, 40];
const EXTRA_SIZES: [i64; 15] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const LIFE_POINTS: [i64; 16] = [
    500, 1000, 1500, 2000, 2500, 3000, 3500, 4000, 4500, 5000, 5500, 6000, 6500, 7000, 7500, 8000,
];
const TURNS: [i64; 19] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
// PHASE_DRAW to PHASE_END by bit, then none.
const PHASES: usize = 11;
// Monster Zones 0-6, then Spell & Trap Zones 0-5 (5 the Field Zone).
const ZONES: usize = 13;
const MONSTER_ZONES: usize = 7;

// Groups indexed per player are the player's (0), then its opponent's (1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Group {
    // Per card row, player and [Place] in that order: every card where the player knows it is, its hidden ones as HIDDEN.
    Card,
    // Per card row: the opponent's cards shown this game, a copy each.
    Shown,
    // Per card row and player: effects activated this turn, an index each (what decides most count limits).
    Activated,
    HandSize,
    DeckSize,
    ExtraSize,
    LifePoints,
    Turn,
    Phase,
    TurnPlayer,
    FirstPlayer,
    // Per player: none or some this turn.
    NormalSummon,
    // Per player and zone: occupied.
    Zone,
    // Per player and Monster Zone: a Link arrow points to it.
    LinkedZone,
}

impl Group {
    pub const ALL: [Self; 14] = [
        Self::Card,
        Self::Shown,
        Self::Activated,
        Self::HandSize,
        Self::DeckSize,
        Self::ExtraSize,
        Self::LifePoints,
        Self::Turn,
        Self::Phase,
        Self::TurnPlayer,
        Self::FirstPlayer,
        Self::NormalSummon,
        Self::Zone,
        Self::LinkedZone,
    ];

    pub const fn size(self, rows: usize) -> usize {
        match self {
            Self::Card => rows * 2 * Place::ALL.len(),
            Self::Shown => rows,
            Self::Activated => rows * 2,
            Self::HandSize => 2 * (HAND_SIZES.len() + 1),
            Self::DeckSize => 2 * (DECK_SIZES.len() + 1),
            Self::ExtraSize => 2 * (EXTRA_SIZES.len() + 1),
            Self::LifePoints => 2 * (LIFE_POINTS.len() + 1),
            Self::Turn => TURNS.len() + 1,
            Self::Phase => PHASES,
            Self::TurnPlayer | Self::FirstPlayer => 2,
            Self::NormalSummon => 2 * 2,
            Self::Zone => 2 * ZONES,
            Self::LinkedZone => 2 * MONSTER_ZONES,
        }
    }
}

// Card codes to rows, [HIDDEN] and [OTHER] first, then the codes in ascending order, so a saved code list gives the same rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vocabulary {
    codes: Vec<u32>,
    starts: [usize; Group::ALL.len()],
    features: usize,
}

impl Vocabulary {
    pub fn new(codes: impl IntoIterator<Item = u32>) -> Self {
        let mut codes: Vec<u32> = codes.into_iter().filter(|&code| code != 0).collect();
        codes.sort_unstable();
        codes.dedup();
        let rows = codes.len() + 2;
        let mut starts = [0; Group::ALL.len()];
        let mut features = 0;
        for (start, group) in starts.iter_mut().zip(Group::ALL) {
            *start = features;
            features += group.size(rows);
        }
        Self { codes, starts, features }
    }

    pub fn codes(&self) -> &[u32] {
        &self.codes
    }

    pub fn rows(&self) -> usize {
        self.codes.len() + 2
    }

    pub fn features(&self) -> usize {
        self.features
    }

    pub fn start(&self, group: Group) -> usize {
        self.starts.get(group as usize).copied().unwrap_or(0)
    }

    pub fn row(&self, code: u32) -> Option<usize> {
        self.codes.binary_search(&code).ok().map(|index| index + 2)
    }
}

struct Bag<'a> {
    vocabulary: &'a Vocabulary,
    duel: &'a Duel,
    player: u8,
    out: &'a mut Vec<u32>,
}

impl Bag<'_> {
    fn push(&mut self, group: Group, index: usize) {
        self.out.push((self.vocabulary.start(group) + index) as u32);
    }

    fn side(&self, controller: u8) -> usize {
        usize::from(controller != self.player)
    }

    // An alternate artwork takes the row of the card it is an alias of.
    fn row(&self, code: u32) -> usize {
        let alias = || self.vocabulary.row(self.duel.database()?.get(code)?.alias);
        match code {
            0 => HIDDEN,
            code => self.vocabulary.row(code).or_else(alias).unwrap_or(OTHER),
        }
    }

    fn card(&mut self, code: u32, controller: u8, place: Place) {
        let index = (self.row(code) * 2 + self.side(controller)) * Place::ALL.len() + place as usize;
        self.push(Group::Card, index);
    }

    fn bucket(&mut self, group: Group, controller: u8, value: i64, edges: &[i64]) {
        let index = self.side(controller) * (edges.len() + 1) + edges.partition_point(|&edge| edge <= value);
        self.push(group, index);
    }
}

// Appends, sorted, the features of what `tracker`'s player knows of `duel`: what [Duel::observe] shows it, what the tracker
// remembers, and public state the observation leaves out.
pub fn features(duel: &Duel, tracker: &Tracker, vocabulary: &Vocabulary, out: &mut Vec<u32>) {
    let start = out.len();
    let player = tracker.player();
    let observation = duel.observe(player);
    let mut bag = Bag {
        vocabulary,
        duel,
        player,
        out,
    };
    let mut unknown = Vec::new();
    for card in &observation.cards {
        let code = if card.code != 0 { card.code } else { tracker.code_at(card) };
        let pile = matches!(
            card.location,
            LOCATION_DECK | LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED | LOCATION_EXTRA
        );
        match code {
            0 if pile => unknown.push((card.controller, card.location, Place::of(card))),
            code => bag.card(code, card.controller, Place::of(card)),
        }
        let zone = match card.location {
            LOCATION_MZONE => Some(usize::from(card.sequence)),
            LOCATION_SZONE => Some(MONSTER_ZONES + usize::from(card.sequence)),
            _ => None,
        };
        if let Some(zone) = zone.filter(|&zone| zone < ZONES) {
            bag.push(Group::Zone, bag.side(card.controller) * ZONES + zone);
        }
    }
    unknown.sort_unstable();
    for pile in unknown.chunk_by(|a, b| (a.0, a.1) == (b.0, b.1)) {
        for (index, &(controller, location, place)) in pile.iter().enumerate() {
            let code = tracker.unplaced(controller, location).get(index).copied().unwrap_or(0);
            bag.card(code, controller, place);
        }
    }
    for (code, count) in tracker.shown() {
        let row = bag.row(code);
        (0..count).for_each(|_| bag.push(Group::Shown, row));
    }
    for controller in [player, 1 - player] {
        let size = |location| {
            (observation.cards.iter())
                .filter(|card| (card.controller, card.location) == (controller, location))
                .count()
        };
        bag.bucket(Group::HandSize, controller, size(LOCATION_HAND) as i64, &HAND_SIZES);
        bag.bucket(Group::DeckSize, controller, size(LOCATION_DECK) as i64, &DECK_SIZES);
        bag.bucket(Group::ExtraSize, controller, size(LOCATION_EXTRA) as i64, &EXTRA_SIZES);
        let life_points = observation.lp.get(usize::from(controller)).copied().unwrap_or(0);
        bag.bucket(Group::LifePoints, controller, life_points.into(), &LIFE_POINTS);
        let side = bag.side(controller);
        bag.push(Group::NormalSummon, side * 2 + usize::from(tracker.normal_summons(controller) > 0));
        for &code in tracker.activated(controller) {
            let row = bag.row(code);
            bag.push(Group::Activated, row * 2 + side);
        }
        let linked = duel.linked_zones(player) >> (16 * side);
        for zone in (0..MONSTER_ZONES).filter(|zone| linked >> zone & 1 != 0) {
            bag.push(Group::LinkedZone, side * MONSTER_ZONES + zone);
        }
    }
    bag.push(Group::Turn, TURNS.partition_point(|&edge| edge <= observation.turn.into()));
    bag.push(Group::Phase, (observation.phase.trailing_zeros() as usize).min(PHASES - 1));
    bag.push(Group::TurnPlayer, bag.side(observation.turn_player));
    let first = observation.turn_player ^ u8::from(observation.turn % 2 == 0);
    bag.push(Group::FirstPlayer, bag.side(first));
    if let Some(bag) = out.get_mut(start..) {
        bag.sort_unstable();
    }
}
