use std::cmp::Ordering;
use std::collections::HashMap;
use std::num::{NonZeroU16, NonZeroU64};
use std::ops::{Deref, Index, IndexMut};

use carddata::Card as CardData;

use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Container, EffectId, EffectSet, Target, Value, container_key, remove_item};
use crate::list::List;
use crate::script::{Args, Procedure};

pub const MAX_CARDS: usize = 512;

// The card's index in creation order, which orders cards as ocgcore's `cardid` does. Card 0 is the core's temp_card,
// the owner of global effects. Stored plus one, so that an Option<CardId> takes no more space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(NonZeroU16);

impl CardId {
    fn new(index: u16) -> Self {
        Self(NonZeroU16::MIN.saturating_add(index))
    }

    pub fn index(self) -> usize {
        usize::from(self.0.get() - 1)
    }
}

impl Default for CardId {
    fn default() -> Self {
        Self::new(0)
    }
}

impl std::fmt::Debug for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CardId").field(&self.index()).finish()
    }
}

// card_set: ordered by cardid like card_sort. Scripts see it as a Group.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardSet([u64; MAX_CARDS / 64]);

impl CardSet {
    pub fn insert(&mut self, card: CardId) {
        if let Some(word) = self.0.get_mut(card.index() / 64) {
            *word |= 1 << (card.index() % 64);
        }
    }

    pub fn remove(&mut self, card: CardId) {
        if let Some(word) = self.0.get_mut(card.index() / 64) {
            *word &= !(1 << (card.index() % 64));
        }
    }

    pub fn contains(&self, card: CardId) -> bool {
        self.0
            .get(card.index() / 64)
            .is_some_and(|word| word & (1 << (card.index() % 64)) != 0)
    }

    pub fn difference(mut self, other: CardSet) -> CardSet {
        self.0.iter_mut().zip(other.0).for_each(|(word, other)| *word &= !other);
        self
    }

    pub fn intersection(mut self, other: CardSet) -> CardSet {
        self.0.iter_mut().zip(other.0).for_each(|(word, other)| *word &= other);
        self
    }

    pub fn union(mut self, other: CardSet) -> CardSet {
        self.0.iter_mut().zip(other.0).for_each(|(word, other)| *word |= other);
        self
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|word| word.count_ones() as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&word| word == 0)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn iter(&self) -> CardSetIter {
        CardSetIter { words: self.0, index: 0 }
    }

    pub fn first(&self) -> Option<CardId> {
        self.iter().next()
    }
}

pub struct CardSetIter {
    words: [u64; MAX_CARDS / 64],
    index: usize,
}

impl Iterator for CardSetIter {
    type Item = CardId;

    fn next(&mut self) -> Option<CardId> {
        loop {
            let word = self.words.get_mut(self.index)?;
            if *word != 0 {
                let bit = word.trailing_zeros();
                *word &= *word - 1;
                return Some(CardId::new((self.index * 64) as u16 + bit as u16));
            }
            self.index += 1;
        }
    }
}

impl IntoIterator for CardSet {
    type Item = CardId;
    type IntoIter = CardSetIter;

    fn into_iter(self) -> CardSetIter {
        self.iter()
    }
}

impl From<CardId> for CardSet {
    fn from(card: CardId) -> Self {
        Self::from_iter([card])
    }
}

impl FromIterator<CardId> for CardSet {
    fn from_iter<I: IntoIterator<Item = CardId>>(cards: I) -> Self {
        let mut set = Self::default();
        cards.into_iter().for_each(|card| set.insert(card));
        set
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cards {
    list: Vec<Card>,
    scratch: Card,
}

impl Cards {
    pub fn copy_from(&mut self, source: &Self) {
        self.list.clone_from(&source.list);
    }

    pub fn push(&mut self, card: Card) -> Option<CardId> {
        let id = CardId::new(u16::try_from(self.list.len()).ok().filter(|&id| usize::from(id) < MAX_CARDS)?);
        self.list.push(card);
        Some(id)
    }

    pub fn get_info_location(&self, card: CardId) -> LocInfo {
        let card = &self[card];
        match card.overlay_target {
            Some(target) => {
                let target = &self[target].current;
                LocInfo {
                    controler: target.controler,
                    location: target.location | LOCATION_OVERLAY,
                    sequence: target.sequence,
                    position: card.current.sequence,
                }
            }
            None => LocInfo {
                controler: card.current.controler,
                location: card.current.location,
                sequence: card.current.sequence,
                position: card.current.position,
            },
        }
    }
}

// Ids are only handed out by `push` and cards are never removed, so the scratch card is never reached.
impl Index<CardId> for Cards {
    type Output = Card;

    fn index(&self, card: CardId) -> &Card {
        self.list.get(card.index()).unwrap_or(&self.scratch)
    }
}

impl IndexMut<CardId> for Cards {
    fn index_mut(&mut self, card: CardId) -> &mut Card {
        let Self { list, scratch } = self;
        list.get_mut(card.index()).unwrap_or(scratch)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocInfo {
    pub controler: u8,
    pub location: u8,
    pub sequence: u32,
    pub position: u32,
}

// The codes get_attack or get_defense reads: its update, set, final, swap final and base codes, and the other stat's base.
struct StatCodes {
    attack: bool,
    update: u32,
    set: u32,
    final_: u32,
    swap_final: u32,
    base: u32,
    other_base: u32,
}

const ATTACK: StatCodes = StatCodes {
    attack: true,
    update: EFFECT_UPDATE_ATTACK,
    set: EFFECT_SET_ATTACK,
    final_: EFFECT_SET_ATTACK_FINAL,
    swap_final: EFFECT_SWAP_ATTACK_FINAL,
    base: EFFECT_SET_BASE_ATTACK,
    other_base: EFFECT_SET_BASE_DEFENSE,
};

const DEFENSE: StatCodes = StatCodes {
    attack: false,
    update: EFFECT_UPDATE_DEFENSE,
    set: EFFECT_SET_DEFENSE,
    final_: EFFECT_SET_DEFENSE_FINAL,
    swap_final: EFFECT_SWAP_DEFENSE_FINAL,
    base: EFFECT_SET_BASE_DEFENSE,
    other_base: EFFECT_SET_BASE_ATTACK,
};

// card_state, reduced to what the port reads so far.
#[derive(Debug, Clone, Copy, Default)]
pub struct CardState {
    pub controler: u8,
    pub location: u8,
    pub sequence: u32,
    pub position: u32,
    pub reason: u32,
    pub pzone: bool,
    pub reason_player: u8,
    pub reason_effect: Option<EffectId>,
    pub reason_card: Option<CardId>,
}

impl CardState {
    pub fn is_location(&self, location: u16) -> bool {
        u16::from(self.location) & location & 0xff != 0
            || (location & LOCATION_EMZONE != 0 && self.location == LOCATION_MZONE && self.sequence >= 5)
            || (location & LOCATION_MMZONE != 0 && self.location == LOCATION_MZONE && self.sequence < 5)
            || (location & LOCATION_STZONE != 0 && self.location == LOCATION_SZONE && self.sequence < 5)
            || (location & LOCATION_FZONE != 0 && self.location == LOCATION_SZONE && self.sequence == 5)
            || (location & LOCATION_PZONE != 0 && self.location == LOCATION_SZONE && self.pzone)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SummonState {
    pub summon_type: u32,
    pub location: u8,
    pub sequence: u32,
    pub pzone: bool,
    pub player: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SendtoParam {
    pub playerid: u8,
    pub position: u8,
    pub location: u8,
    pub sequence: u8,
}

// The read-only data of a card in the database, which every duel shares instead of copying it into its cards.
#[derive(Debug, Clone, Copy)]
pub struct StaticData(&'static CardData);

// What the core reads for code 0.
static BLANK: CardData = CardData {
    code: 0,
    alias: 0,
    ot: 0,
    card_type: 0,
    level: 0,
    lscale: 0,
    rscale: 0,
    attribute: 0,
    race: 0,
    attack: 0,
    defense: 0,
    link_marker: 0,
    setcodes: [0; 5],
};

impl Default for StaticData {
    fn default() -> Self {
        Self(&BLANK)
    }
}

impl StaticData {
    // Data no database holds (the stats Duel.IsPlayerCanSpecialSummonMonster gives the temp card), kept for the life of
    // the thread: each distinct value is leaked once.
    pub fn interned(data: CardData) -> Self {
        thread_local! {
            static INTERNED: std::cell::RefCell<HashMap<CardData, &'static CardData>> = std::cell::RefCell::default();
        }
        Self(INTERNED.with_borrow_mut(|interned| *interned.entry(data).or_insert_with(|| Box::leak(Box::new(data)))))
    }
}

impl From<&'static CardData> for StaticData {
    fn from(data: &'static CardData) -> Self {
        Self(data)
    }
}

impl Deref for StaticData {
    type Target = CardData;

    fn deref(&self) -> &CardData {
        self.0
    }
}

// card in card.h. Copy, so the card list clones as one memcpy; its effect containers and relations live in the duel.
#[derive(Debug, Clone, Copy, Default)]
pub struct Card {
    pub data: StaticData,
    pub previous: CardState,
    pub temp: CardState,
    pub current: CardState,
    pub owner: u8,
    pub status: u32,
    pub summon: SummonState,
    pub sendto_param: SendtoParam,
    pub to_field_param: u32,
    pub position_param: u32,
    pub release_param: u32,
    pub spsummon_param: u32,
    pub spsummon_code: u32,
    pub attack_announce_count: u8,
    pub announce_count: u8,
    pub attacked_count: u8,
    pub attack_all_target: u8,
    pub direct_attackable: u8,
    pub attack_controler: u8,
    pub fieldid: u32,
    pub fieldid_r: u32,
    pub turnid: u16,
    pub overlay_target: Option<CardId>,
    pub pre_overlay_target: Option<CardId>,
    pub equiping_target: Option<CardId>,
}

impl Card {
    pub fn new(data: StaticData) -> Self {
        Self {
            data,
            owner: PLAYER_NONE,
            current: CardState {
                controler: PLAYER_NONE,
                ..CardState::default()
            },
            ..Self::default()
        }
    }

    pub fn is_position(&self, position: u32) -> bool {
        self.current.position & position != 0
    }

    pub fn set_status(&mut self, status: u32, enabled: bool) {
        if enabled {
            self.status |= status;
        } else {
            self.status &= !status;
        }
    }

    pub fn get_status(&self, status: u32) -> bool {
        self.status & status != 0
    }

    pub fn is_status(&self, status: u32) -> bool {
        self.status & status == status
    }
}

// A card's relation to a chain link (relate_effect in card.h): the chain's effect and id, or id 0 for an effect.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Relation {
    pub card: CardId,
    pub effect: EffectId,
    pub chain_id: u16,
}

const CARD_CONTAINERS: [Container; 5] = [
    Container::Single,
    Container::Field,
    Container::Equip,
    Container::Target,
    Container::XMaterial,
];

impl Duel {
    pub(crate) fn card_operation_sort(&self, c1: CardId, c2: CardId) -> Ordering {
        let less = |c1: CardId, c2: CardId| {
            let (card1, card2) = (&self.cards[c1], &self.cards[c2]);
            let controler = |card: &Card| {
                card.overlay_target
                    .map_or(card.current.controler, |target| self.cards[target].current.controler)
            };
            let (cp1, cp2) = (controler(card1), controler(card2));
            if cp1 != cp2 {
                return if cp1 == PLAYER_NONE || cp2 == PLAYER_NONE || self.infos.turn_player == 0 {
                    cp1 < cp2
                } else {
                    cp1 > cp2
                };
            }
            if card1.current.location != card2.current.location {
                return card1.current.location < card2.current.location;
            }
            if card1.current.location & LOCATION_OVERLAY != 0 {
                let sequence = |card: &Card| card.overlay_target.map_or(0, |target| self.cards[target].current.sequence);
                if sequence(card1) != sequence(card2) {
                    return sequence(card1) < sequence(card2);
                }
                card1.current.sequence < card2.current.sequence
            } else if card1.current.location & (LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                card1.current.sequence > card2.current.sequence
            } else {
                card1.current.sequence < card2.current.sequence
            }
        };
        if less(c1, c2) {
            Ordering::Less
        } else if less(c2, c1) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    pub(crate) fn is_extra_deck_monster(&self, card: CardId) -> bool {
        let data = &self.cards[card].data;
        data.card_type & TYPE_MONSTER != 0 && data.card_type & self.get_extra_deck_types() != 0
    }

    // card::get_code: the alias for alternate arts and the last EFFECT_CHANGE_CODE; EFFECT_ADD_CODE comes with its cards.
    pub(crate) fn get_code(&self, card: CardId) -> u32 {
        let data = &self.cards[card].data;
        let code = self
            .filter_effect(card, EFFECT_CHANGE_CODE, true)
            .last()
            .map_or(data.code, |&effect| self.get_value(effect, card) as u32);
        if code == data.code {
            unported!(self, self.filter_effect(card, EFFECT_ADD_CODE, true));
            return if data.alias != 0 { data.alias } else { code };
        }
        match self.database.and_then(|database| database.get(code)) {
            Some(changed) if changed.alias != 0 && Self::second_code(code) == 0 => changed.alias,
            _ => code,
        }
    }

    // card::second_code: the mapping of double-name cards.
    fn second_code(code: u32) -> u32 {
        match code {
            78_734_254 => 17_955_766,
            13_857_930 => 17_732_278,
            _ => 0,
        }
    }

    pub(crate) fn get_another_code(&self, card: CardId) -> u32 {
        let code = self.get_code(card);
        if code != self.cards[card].data.code {
            return Self::second_code(code);
        }
        unported!(self, self.filter_effect(card, EFFECT_ADD_CODE, true));
        0
    }

    pub(crate) fn get_type(&self, card: CardId) -> u32 {
        self.get_type_for(card, None, 0, PLAYER_NONE)
    }

    // card::get_type(scard, sumtype, playerid): the type as material of `scard` summoned by `sumtype`. Type effects
    // with an operation, which apply only to such checks, come with the cards that register them.
    #[inline(always)]
    pub(crate) fn get_type_for(&self, card: CardId, _scard: Option<CardId>, sumtype: u64, _playerid: u8) -> u32 {
        let current = self.cards[card].current;
        let data_type = self.cards[card].data.card_type;
        if current.location & (LOCATION_ONFIELD | LOCATION_HAND | LOCATION_GRAVE) == 0 {
            return data_type;
        }
        if current.is_location(LOCATION_PZONE) && sumtype == 0 {
            return TYPE_PENDULUM + TYPE_SPELL;
        }
        let alttype = if sumtype != 0 && current.location == LOCATION_SZONE {
            data_type
        } else {
            0
        };
        let registry = &self.field_effects.registry;
        if ![EFFECT_ADD_TYPE, EFFECT_REMOVE_TYPE, EFFECT_CHANGE_TYPE]
            .iter()
            .any(|&code| registry.has_code(code))
        {
            return data_type | alttype;
        }
        self.get_type_lookup(card, data_type) | alttype
    }

    fn get_type_lookup(&self, card: CardId, data_type: u32) -> u32 {
        let mut card_type = data_type;
        for effect in self.filter_effects(card, &[EFFECT_ADD_TYPE, EFFECT_REMOVE_TYPE, EFFECT_CHANGE_TYPE]) {
            unported!(self, self.effects[effect].operation.map(|_| effect));
            let value = self.get_value(effect, card) as u32;
            match self.effects[effect].code {
                EFFECT_ADD_TYPE => card_type |= value,
                EFFECT_REMOVE_TYPE => card_type &= !value,
                _ => card_type = value,
            }
        }
        card_type
    }

    pub(crate) fn get_level(&self, card: CardId) -> u32 {
        let (data, status) = (*self.cards[card].data, self.cards[card].status);
        if (data.card_type & TYPE_XYZ != 0
            && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL).is_none()
            && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL_S).is_none())
            || data.card_type & TYPE_LINK != 0
            || status & STATUS_NO_LEVEL != 0
            || (data.card_type & TYPE_MONSTER == 0
                && self.get_type(card) & TYPE_MONSTER == 0
                && self.is_affected_by_effect(card, EFFECT_PRE_MONSTER).is_none())
        {
            return 0;
        }
        let mut level = data.level as i32;
        unported!(
            self,
            [EFFECT_RANK_LEVEL_S, EFFECT_LEVEL_RANK_S]
                .into_iter()
                .filter_map(|code| self.is_affected_by_effect(card, code))
        );
        let (mut up, mut upc) = (0, 0);
        for effect in self.filter_effects(card, &[EFFECT_UPDATE_LEVEL, EFFECT_CHANGE_LEVEL, EFFECT_CHANGE_LEVEL_FINAL]) {
            let value = self.get_value(effect, card);
            match self.effects[effect].code {
                EFFECT_UPDATE_LEVEL if self.is_single_unranged(effect) => up += value,
                EFFECT_UPDATE_LEVEL => upc += value,
                EFFECT_CHANGE_LEVEL => (level, up) = (value, 0),
                _ => (level, up, upc) = (value, 0, 0),
            }
        }
        level += up + upc;
        if level < 1 && self.get_type(card) & TYPE_MONSTER != 0 && self.is_affected_by_effect(card, EFFECT_ALLOW_NEGATIVE).is_none() {
            level = 1;
        }
        level as u32
    }

    pub(crate) fn get_rank(&self, card: CardId) -> u32 {
        let (data, status) = (*self.cards[card].data, self.cards[card].status);
        if ((data.card_type & TYPE_XYZ == 0 || status & STATUS_NO_LEVEL != 0)
            && self.is_affected_by_effect(card, EFFECT_LEVEL_RANK).is_none()
            && self.is_affected_by_effect(card, EFFECT_LEVEL_RANK_S).is_none())
            || data.card_type & TYPE_LINK != 0
        {
            return 0;
        }
        if self.cards[card].current.location & LOCATION_MZONE == 0 {
            return data.level;
        }
        unported!(
            self,
            [EFFECT_RANK_LEVEL_S, EFFECT_LEVEL_RANK_S]
                .into_iter()
                .filter_map(|code| self.is_affected_by_effect(card, code))
        );
        let mut rank = data.level as i32;
        let (mut up, mut upc) = (0, 0);
        for effect in self.filter_effects(card, &[EFFECT_UPDATE_RANK, EFFECT_CHANGE_RANK, EFFECT_CHANGE_RANK_FINAL]) {
            let value = self.get_value(effect, card);
            match self.effects[effect].code {
                EFFECT_UPDATE_RANK if self.is_single_unranged(effect) => up += value,
                EFFECT_UPDATE_RANK => upc += value,
                EFFECT_CHANGE_RANK => (rank, up) = (value, 0),
                _ => (rank, up, upc) = (value, 0, 0),
            }
        }
        rank += up + upc;
        if rank < 1 && self.get_type(card) & TYPE_MONSTER != 0 && self.is_affected_by_effect(card, EFFECT_ALLOW_NEGATIVE).is_none() {
            rank = 1;
        }
        rank as u32
    }

    // card::check_xyz_level; EFFECT_XYZ_LEVEL comes with the cards that register it.
    pub(crate) fn check_xyz_level(&self, card: CardId, _xyzc: CardId, level: u32) -> bool {
        if self.cards[card].status & STATUS_NO_LEVEL != 0 {
            return false;
        }
        unported!(self, self.filter_effect(card, EFFECT_XYZ_LEVEL, true));
        self.get_level(card) == level
    }

    pub(crate) fn get_attribute(&self, card: CardId) -> u32 {
        if self.cards[card].data.card_type & TYPE_MONSTER == 0
            && self.get_type(card) & TYPE_MONSTER == 0
            && self.is_affected_by_effect(card, EFFECT_PRE_MONSTER).is_none()
        {
            return 0;
        }
        let mut attribute = self.cards[card].data.attribute;
        for effect in self.filter_effects(card, &[EFFECT_ADD_ATTRIBUTE, EFFECT_REMOVE_ATTRIBUTE, EFFECT_CHANGE_ATTRIBUTE]) {
            let peffect = &self.effects[effect];
            unported!(self, peffect.operation.is_some().then_some(effect));
            let value = self.get_value(effect, card) as u32;
            match peffect.code {
                EFFECT_ADD_ATTRIBUTE => attribute |= value,
                EFFECT_REMOVE_ATTRIBUTE => attribute &= !value,
                _ => attribute = value,
            }
        }
        attribute
    }

    pub(crate) fn get_race(&self, card: CardId) -> u64 {
        let assumed = self
            .assumes
            .borrow()
            .iter()
            .find(|&&(other, kind, _)| (other, u32::from(kind)) == (card, ASSUME_RACE))
            .map(|&(_, _, race)| race);
        if let Some(race) = assumed {
            return race;
        }
        if self.cards[card].data.card_type & TYPE_MONSTER == 0
            && self.get_type(card) & TYPE_MONSTER == 0
            && self.is_affected_by_effect(card, EFFECT_PRE_MONSTER).is_none()
        {
            return 0;
        }
        unported!(
            self,
            [EFFECT_ADD_RACE, EFFECT_REMOVE_RACE, EFFECT_CHANGE_RACE]
                .into_iter()
                .flat_map(|code| self.filter_effect(card, code, true))
        );
        self.cards[card].data.race
    }

    // get_lscale and get_rscale.
    pub(crate) fn get_scale(&self, card: CardId, right: bool) -> u32 {
        let data = &self.cards[card].data;
        let scale = if right { data.rscale } else { data.lscale };
        if !self.cards[card].current.is_location(LOCATION_PZONE) {
            return scale;
        }
        let codes = if right {
            [EFFECT_UPDATE_RSCALE, EFFECT_CHANGE_RSCALE]
        } else {
            [EFFECT_UPDATE_LSCALE, EFFECT_CHANGE_LSCALE]
        };
        unported!(self, codes.into_iter().flat_map(|code| self.filter_effect(card, code, true)));
        scale
    }

    // card::get_set_card with match_setcode: whether one of the card's archetypes matches `setcode`.
    pub(crate) fn is_set_card(&self, card: CardId, setcode: u16) -> bool {
        let matches = |code: u16| code != 0 && (setcode & 0xfff) == (code & 0xfff) && (setcode & code) == setcode;
        self.get_set_card(card).iter().any(|&code| matches(code))
    }

    // card::get_set_card: the card's archetypes, 0 in unused slots.
    pub(crate) fn get_set_card(&self, card: CardId) -> [u16; 5] {
        unported!(self, self.filter_effect(card, EFFECT_ADD_SETCODE, true));
        let data = &self.cards[card].data;
        let code = self.get_code(card);
        let own = if code == data.code {
            data.setcodes
        } else {
            self.database
                .and_then(|database| database.get(code))
                .map_or([0; 5], |changed| changed.setcodes)
        };
        unported!(self, (self.get_another_code(card) != 0).then_some(()));
        own
    }

    // The effects of `codes` on the card, in id order, as consecutive filter_effect calls collect them.
    pub(crate) fn filter_effects(&self, card: CardId, codes: &[u32]) -> EffectSet {
        let mut effects = EffectSet::default();
        for &code in codes {
            if self.field_effects.registry.has_code(code) {
                self.filter_effect_into(card, code, &mut effects);
            }
        }
        if effects.len() > 1 {
            effects.sort_by_key(|&effect| self.effects.id(effect));
        }
        effects
    }

    // A single effect without EFFECT_FLAG_SINGLE_RANGE, which the stat functions add up apart from the others.
    fn is_single_unranged(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        peffect.effect_type & EFFECT_TYPE_SINGLE != 0 && !peffect.is_flag(EFFECT_FLAG_SINGLE_RANGE)
    }

    pub(crate) fn get_attack(&self, card: CardId) -> i32 {
        self.get_stat_value(card, &ATTACK, false)
    }

    // The `code` (EFFECT_SET_BASE_ATTACK or _DEFENSE) effects other than a card's own unranged ones apply first, in id
    // order; returns the base they leave and the other effects.
    fn continuous_base(&self, card: CardId, code: u32, base: i32, effects: EffectSet) -> (i32, EffectSet) {
        let mut base = base;
        let mut rest = EffectSet::default();
        for effect in effects {
            if self.effects[effect].code == code && !self.is_single_unranged(effect) {
                base = self.get_value(effect, card).max(0);
            } else {
                rest.push(effect);
            }
        }
        (base, rest)
    }

    pub(crate) fn get_base_attack(&self, card: CardId) -> i32 {
        let swaps = [EFFECT_SWAP_BASE_AD, EFFECT_SET_BASE_DEFENSE];
        self.get_base_stat(card, EFFECT_SET_BASE_ATTACK, &swaps, self.cards[card].data.attack)
    }

    pub(crate) fn get_base_defense(&self, card: CardId) -> i32 {
        if self.cards[card].data.card_type & TYPE_LINK != 0 {
            return 0;
        }
        self.get_base_stat(card, EFFECT_SET_BASE_DEFENSE, &[EFFECT_SWAP_BASE_AD], self.cards[card].data.defense)
    }

    fn get_base_stat(&self, card: CardId, code: u32, swaps: &[u32], base: i32) -> i32 {
        let base = self.get_stat(card, &[], swaps, base);
        let target = &self.cards[card];
        if target.current.location != LOCATION_MZONE || target.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP) {
            return base;
        }
        let (base, effects) = self.continuous_base(card, code, base, self.filter_effects(card, &[code]));
        effects.iter().fold(base, |_, &effect| self.get_value(effect, card).max(0))
    }

    pub(crate) fn get_defense(&self, card: CardId) -> i32 {
        if self.cards[card].data.card_type & TYPE_LINK != 0 {
            return 0;
        }
        self.get_stat_value(card, &DEFENSE, false)
    }

    // card::get_attack and card::get_defense, which differ only in the codes they read. `nested` is the call the swap
    // of EFFECT_SWAP_AD makes, which the core's temp values stop from swapping back. The ASSUME_* overrides and the
    // temp values a value function reading the same stat would get come with the cards that need them.
    fn get_stat_value(&self, card: CardId, stat: &StatCodes, nested: bool) -> i32 {
        let target = &self.cards[card];
        if target.data.card_type & TYPE_MONSTER == 0
            && self.get_type(card) & TYPE_MONSTER == 0
            && self.is_affected_by_effect(card, EFFECT_PRE_MONSTER).is_none()
        {
            return 0;
        }
        let (own, other) = if stat.attack {
            (target.data.attack, target.data.defense)
        } else {
            (target.data.defense, target.data.attack)
        };
        if target.current.location != LOCATION_MZONE || target.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP) {
            return own;
        }
        let (mut base, mut other_base) = (own.max(0), other.max(0));
        let mut codes: List<u32, 8> = [stat.update, stat.set, stat.final_, stat.swap_final, stat.base]
            .into_iter()
            .collect();
        if target.data.card_type & TYPE_LINK == 0 {
            codes.extend([EFFECT_SWAP_AD, EFFECT_SWAP_BASE_AD, stat.other_base]);
        }
        let rev = self.is_affected_by_effect(card, EFFECT_REVERSE_UPDATE).is_some();
        let mut effects = EffectSet::default();
        for effect in self.filter_effects(card, &codes) {
            let code = self.effects[effect].code;
            if !self.is_single_unranged(effect) && (code == stat.base || code == stat.other_base) {
                let value = self.get_value(effect, card).max(0);
                *(if code == stat.base { &mut base } else { &mut other_base }) = value;
            } else {
                effects.push(effect);
            }
        }
        let mut value = base;
        let (mut set, mut up, mut upc, mut swap_final) = (-1, 0, 0, false);
        let (mut finals, mut delayed_finals) = (EffectSet::default(), EffectSet::default());
        for effect in effects {
            let peffect = &self.effects[effect];
            match peffect.code {
                code if code == stat.update && self.is_single_unranged(effect) => up += self.get_value(effect, card),
                code if code == stat.update => upc += self.get_value(effect, card),
                code if code == stat.set => {
                    set = self.get_value(effect, card);
                    if peffect.effect_type & EFFECT_TYPE_SINGLE == 0 {
                        up = 0;
                    }
                }
                code if code == stat.final_ && self.is_single_unranged(effect) => {
                    (set, up, upc) = (self.get_value(effect, card), 0, 0);
                }
                code if code == stat.final_ && peffect.is_flag(EFFECT_FLAG_DELAY) => delayed_finals.push(effect),
                code if code == stat.final_ => finals.push(effect),
                code if code == stat.base => (base, set) = (self.get_value(effect, card).max(0), -1),
                code if code == stat.swap_final => (set, up, upc) = (self.get_value(effect, card), 0, 0),
                code if code == stat.other_base => other_base = self.get_value(effect, card).max(0),
                EFFECT_SWAP_AD => swap_final = !swap_final,
                EFFECT_SWAP_BASE_AD => std::mem::swap(&mut base, &mut other_base),
                _ => {}
            }
            let start = if set < 0 { base } else { set };
            value = if rev { start - up - upc } else { start + up + upc }.max(0);
        }
        for effect in finals {
            value = self.get_value(effect, card);
        }
        if !nested {
            if swap_final {
                value = self.get_stat_value(card, if stat.attack { &DEFENSE } else { &ATTACK }, true);
            }
            for effect in delayed_finals {
                value = self.get_value(effect, card);
                if self.effects[effect].is_flag(EFFECT_FLAG_REPEAT) {
                    value = self.get_value(effect, card);
                }
            }
        }
        value.max(0)
    }

    // The checks get_base_attack starts with; `swaps` are skipped for Link monsters.
    fn get_stat(&self, card: CardId, codes: &[u32], swaps: &[u32], base: i32) -> i32 {
        let target = &self.cards[card];
        if target.data.card_type & TYPE_MONSTER == 0
            && self.get_type(card) & TYPE_MONSTER == 0
            && self.is_affected_by_effect(card, EFFECT_PRE_MONSTER).is_none()
        {
            return 0;
        }
        if target.current.location != LOCATION_MZONE || target.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP) {
            return base;
        }
        let swaps = if target.data.card_type & TYPE_LINK != 0 { &[][..] } else { swaps };
        unported!(
            self,
            codes.iter().chain(swaps).flat_map(|&code| self.filter_effect(card, code, true))
        );
        unported!(self, self.is_affected_by_effect(card, EFFECT_REVERSE_UPDATE));
        base.max(0)
    }

    pub(crate) fn apply_field_effect(&mut self, card: CardId) {
        if self.cards[card].current.controler == PLAYER_NONE {
            return;
        }
        for effect in self.registered_effects(Container::Field, card) {
            if self.in_field_range(effect, card) {
                self.add_effect(effect, PLAYER_NONE);
            }
        }
    }

    pub(crate) fn cancel_field_effect(&mut self, card: CardId) {
        if self.cards[card].current.controler == PLAYER_NONE {
            return;
        }
        for effect in self.registered_effects(Container::Field, card) {
            if self.in_field_range(effect, card) {
                self.remove_effect(effect);
            }
        }
    }

    // Whether a card's field effect is registered on the field: in range, or an optional trigger of the hand.
    fn in_field_range(&self, effect: EffectId, card: CardId) -> bool {
        let peffect = &self.effects[effect];
        self.in_range(effect, card)
            || (peffect.range & u16::from(LOCATION_HAND) != 0
                && peffect.effect_type & EFFECT_TYPE_TRIGGER_O != 0
                && peffect.code & EVENT_PHASE == 0)
    }

    // A copy of a card container, for loops whose body may change the containers.
    pub(crate) fn registered_effects(&self, container: Container, card: CardId) -> EffectSet {
        self.field_effects
            .registry
            .all(container, Some(card))
            .map(|(_, effect)| effect)
            .collect()
    }

    // The effects registered on the card in any container (card::indexer), as a copy.
    pub(crate) fn card_effects(&self, card: CardId) -> List<EffectId, 32> {
        let registry = &self.field_effects.registry;
        let mut effects = List::default();
        for container in CARD_CONTAINERS {
            effects.extend(registry.all(container, Some(card)).map(|(_, effect)| effect));
        }
        effects
    }

    // STATUS_EFFECT_ENABLED: the card is ready to use; false while moving, summoning or chaining.
    pub(crate) fn enable_field_effect(&mut self, card: CardId, enabled: bool) {
        let target = &self.cards[card];
        if target.current.location == 0 || enabled == target.get_status(STATUS_EFFECT_ENABLED) {
            return;
        }
        self.refresh_disable_status(card);
        if enabled {
            self.cards[card].set_status(STATUS_EFFECT_ENABLED, true);
            let registry = &self.field_effects.registry;
            for (_, effect) in registry.all(Container::Single, Some(card)) {
                if self.effects[effect].is_flag(EFFECT_FLAG_SINGLE_RANGE) && self.in_range(effect, card) {
                    self.effects.set_id(effect, self.next_field_id());
                }
            }
            for (_, effect) in registry.all(Container::Field, Some(card)) {
                if self.in_range(effect, card) {
                    self.effects.set_id(effect, self.next_field_id());
                }
            }
            if self.cards[card].current.location == LOCATION_SZONE {
                for (_, effect) in registry.all(Container::Equip, Some(card)) {
                    self.effects.set_id(effect, self.next_field_id());
                }
            }
            for (_, effect) in registry.all(Container::Target, Some(card)) {
                if self.in_range(effect, card) {
                    self.effects.set_id(effect, self.next_field_id());
                }
            }
            if self.cards[card].get_status(STATUS_DISABLED) {
                self.reset(card, RESET_DISABLE, RESET_EVENT);
            }
        } else {
            self.cards[card].set_status(STATUS_EFFECT_ENABLED, false);
        }
        if self.cards[card].get_status(STATUS_DISABLED | STATUS_FORBIDDEN) {
            return;
        }
        self.filter_disable_related_cards(card);
    }

    // card::add_effect; returns the effect's id, or 0 if it was not added.
    pub(crate) fn card_add_effect(&mut self, card: CardId, effect: EffectId) -> u32 {
        let peffect = &self.effects[effect];
        if self.cards[card].get_status(STATUS_COPYING_EFFECT) && peffect.is_flag(EFFECT_FLAG_UNCOPYABLE) {
            return 0;
        }
        if peffect.card_key.is_some() {
            return 0;
        }
        let (effect_type, code) = (peffect.effect_type, peffect.code);
        let mut check_target = Some(card);
        let container = if effect_type & EFFECT_TYPE_SINGLE != 0 {
            // A new stat-setting effect replaces the ones it overrides.
            let replaced: &[u32] = match code {
                _ if peffect.is_flag(EFFECT_FLAG_SINGLE_RANGE) => &[],
                EFFECT_SET_ATTACK | EFFECT_SET_BASE_ATTACK => &[EFFECT_SET_ATTACK, EFFECT_SET_ATTACK_FINAL, EFFECT_SET_BASE_ATTACK],
                EFFECT_SET_ATTACK_FINAL => &[EFFECT_UPDATE_ATTACK, EFFECT_SET_ATTACK, EFFECT_SET_ATTACK_FINAL],
                EFFECT_SET_DEFENSE | EFFECT_SET_BASE_DEFENSE => &[EFFECT_SET_DEFENSE, EFFECT_SET_DEFENSE_FINAL, EFFECT_SET_BASE_DEFENSE],
                EFFECT_SET_DEFENSE_FINAL => &[EFFECT_UPDATE_DEFENSE, EFFECT_SET_DEFENSE, EFFECT_SET_DEFENSE_FINAL],
                _ => &[],
            };
            if !replaced.is_empty() {
                let removed: Vec<EffectId> = self
                    .field_effects
                    .registry
                    .all(Container::Single, Some(card))
                    .filter(|&(code, other)| replaced.contains(&code) && !self.effects[other].is_flag(EFFECT_FLAG_SINGLE_RANGE))
                    .map(|(_, other)| other)
                    .collect();
                for other in removed {
                    self.card_remove_effect(card, other);
                }
            }
            Container::Single
        } else if effect_type & EFFECT_TYPE_EQUIP != 0 {
            check_target = self.cards[card].equiping_target;
            Container::Equip
        } else if effect_type & EFFECT_TYPE_TARGET != 0 {
            check_target = None;
            Container::Target
        } else if effect_type & EFFECT_TYPE_XMATERIAL != 0 {
            // The effects a material gives its Xyz Monster come with the cards that register them; until then every
            // lookup of the materials' xmaterial_effect finds nothing.
            self.reach_unported(concat!(file!(), ":", line!()));
            check_target = self.cards[card].overlay_target;
            Container::XMaterial
        } else if effect_type & EFFECT_TYPE_FIELD != 0 {
            Container::Field
        } else {
            return 0;
        };
        let key = container_key(container, Some(card), code);
        self.field_effects.registry.insert(key, effect);
        if code == EFFECT_SELF_TOGRAVE {
            self.core.global_flag |= GLOBALFLAG_SELF_TOGRAVE;
        } else if code == EVENT_DETACH_MATERIAL {
            self.core.global_flag |= GLOBALFLAG_DETACH_EVENT;
        }
        let id = self.next_field_id();
        let data_type = self.cards[card].data.card_type;
        let initializing = self.cards[card].get_status(STATUS_INITIALIZING);
        self.effects.set_id(effect, id);
        let peffect = &mut self.effects[effect];
        peffect.initial_id = id;
        peffect.card_type = data_type;
        if initializing {
            peffect.flag[0] |= EFFECT_FLAG_INITIAL;
        }
        peffect.card_key = NonZeroU64::new(key);
        peffect.handler = Some(card);
        let reason_effect = self.core.reason_effect.get();
        let (reset_flag, flag) = (peffect.reset_flag, peffect.flag[0]);
        unported!(
            self,
            (flag & EFFECT_FLAG_COPY_INHERIT != 0 && reason_effect.is_some()).then_some(effect)
        );
        if effect_type & EFFECT_TYPE_FIELD != 0 {
            let controler = self.cards[card].current.controler;
            if self.in_range(effect, card)
                || (controler != PLAYER_NONE
                    && self.effects[effect].range & u16::from(LOCATION_HAND) != 0
                    && effect_type & EFFECT_TYPE_TRIGGER_O != 0
                    && code & EVENT_PHASE == 0)
            {
                self.add_effect(effect, PLAYER_NONE);
            }
        }
        if self.cards[card].current.controler != PLAYER_NONE
            && let Some(target) = check_target
            && self.effects[effect].is_disable_related()
        {
            self.add_to_disable_check_list(target);
        }
        if flag & EFFECT_FLAG_OATH != 0 {
            self.field_effects.oath.push((effect, reason_effect));
        }
        if reset_flag & RESET_PHASE != 0 {
            self.field_effects.pheff.push(effect);
            if self.effects[effect].reset_count == 0 {
                self.effects[effect].reset_count += 1;
            }
        }
        if reset_flag & RESET_CHAIN != 0 {
            self.field_effects.cheff.push(effect);
        }
        if flag & EFFECT_FLAG_COUNT_LIMIT != 0 {
            self.field_effects.rechargeable.push(effect);
        }
        if flag & EFFECT_FLAG_CLIENT_HINT != 0 {
            let (location, description) = (self.cards.get_info_location(card), self.effects[effect].description);
            self.trace.message(MSG_CARD_HINT).loc(location).u8(CHINT_DESC_ADD).u64(description);
        }
        if effect_type & EFFECT_TYPE_SINGLE != 0 && code == EFFECT_UPDATE_LEVEL && flag & EFFECT_FLAG_SINGLE_RANGE == 0 {
            let value = self.get_value(effect, card);
            if value > 0 {
                self.field_raise_single_event(card, EVENT_LEVEL_UP, Some(effect), 0, 0, 0, value as u32);
                self.process_single_event();
            }
        }
        id
    }

    pub(crate) fn card_remove_effect(&mut self, card: CardId, effect: EffectId) {
        let Some(key) = self.effects[effect].card_key.map(NonZeroU64::get) else {
            return;
        };
        let peffect = &self.effects[effect];
        let (effect_type, flag, reset_flag) = (peffect.effect_type, peffect.flag[0], peffect.reset_flag);
        let mut check_target = Some(card);
        if effect_type & EFFECT_TYPE_SINGLE != 0 {
        } else if effect_type & EFFECT_TYPE_EQUIP != 0 {
            check_target = self.cards[card].equiping_target;
        } else if effect_type & EFFECT_TYPE_TARGET != 0 {
            check_target = None;
        } else if effect_type & EFFECT_TYPE_XMATERIAL != 0 {
            check_target = self.cards[card].overlay_target;
        } else if effect_type & EFFECT_TYPE_FIELD != 0 {
            check_target = None;
            if self.is_available(effect) && self.effects[effect].is_disable_related() {
                self.update_disable_check_list(effect);
            }
        }
        self.field_effects.registry.remove(key, effect);
        self.effects[effect].card_key = None;
        if effect_type & EFFECT_TYPE_FIELD != 0
            && effect_type & (EFFECT_TYPE_SINGLE | EFFECT_TYPE_EQUIP | EFFECT_TYPE_TARGET | EFFECT_TYPE_XMATERIAL) == 0
            && self.in_field_range_of_removal(effect, card)
        {
            self.remove_effect(effect);
        }
        if self.cards[card].current.controler != PLAYER_NONE
            && let Some(target) = check_target
            && self.effects[effect].is_disable_related()
        {
            self.add_to_disable_check_list(target);
        }
        if flag & EFFECT_FLAG_OATH != 0 {
            self.field_effects.oath.retain(|&(oath, _)| oath != effect);
        }
        if reset_flag & RESET_PHASE != 0 {
            remove_item(&mut self.field_effects.pheff, &effect);
        }
        if reset_flag & RESET_CHAIN != 0 {
            remove_item(&mut self.field_effects.cheff, &effect);
        }
        if flag & EFFECT_FLAG_COUNT_LIMIT != 0 {
            remove_item(&mut self.field_effects.rechargeable, &effect);
        }
        let code = self.effects[effect].code;
        if code & 0xf_0000 == EFFECT_COUNTER_PERMIT && effect_type & EFFECT_TYPE_SINGLE != 0 {
            self.remove_permitted_counters(card, code as u16);
        }
        if flag & EFFECT_FLAG_CLIENT_HINT != 0 {
            let (location, description) = (self.cards.get_info_location(card), self.effects[effect].description);
            self.trace
                .message(MSG_CARD_HINT)
                .loc(location)
                .u8(CHINT_DESC_REMOVE)
                .u64(description);
        }
        self.mark_reseted(effect);
    }

    fn in_field_range_of_removal(&self, effect: EffectId, card: CardId) -> bool {
        let peffect = &self.effects[effect];
        self.in_range(effect, card)
            || (self.cards[card].current.controler != PLAYER_NONE
                && peffect.range & u16::from(LOCATION_HAND) != 0
                && peffect.effect_type & EFFECT_TYPE_TRIGGER_O != 0
                && peffect.code & EVENT_PHASE == 0)
    }

    pub(crate) fn filter_disable_related_cards(&mut self, card: CardId) {
        for effect in self.card_effects(card) {
            {
                if !self.effects[effect].is_disable_related() {
                    continue;
                }
                let effect_type = self.effects[effect].effect_type;
                if effect_type & EFFECT_TYPE_FIELD != 0 {
                    self.update_disable_check_list(effect);
                } else if effect_type & EFFECT_TYPE_EQUIP != 0 {
                    if let Some(target) = self.cards[card].equiping_target {
                        self.add_to_disable_check_list(target);
                    }
                } else if effect_type & EFFECT_TYPE_XMATERIAL != 0
                    && let Some(target) = self.cards[card].overlay_target
                {
                    self.add_to_disable_check_list(target);
                }
            }
        }
    }

    // Refreshes STATUS_DISABLED and STATUS_FORBIDDEN from EFFECT_DISABLE, EFFECT_CANNOT_DISABLE and EFFECT_FORBIDDEN.
    pub(crate) fn refresh_disable_status(&mut self, card: CardId) {
        let forbidden = self.is_affected_by_effect(card, EFFECT_FORBIDDEN).is_some();
        let disabled =
            self.is_affected_by_effect(card, EFFECT_CANNOT_DISABLE).is_none() && self.is_affected_by_effect(card, EFFECT_DISABLE).is_some();
        self.immune_effects.retain(|(cached, _)| *cached != card);
        let immune_effects = self.filter_immune_effect(card);
        if !immune_effects.is_empty() {
            self.immune_effects.push((card, immune_effects));
        }
        let card = &mut self.cards[card];
        card.set_status(STATUS_FORBIDDEN, forbidden);
        card.set_status(STATUS_DISABLED, disabled);
    }

    pub(crate) fn reset(&mut self, card: CardId, id: u32, reset_type: u32) {
        if ![RESET_EVENT, RESET_PHASE, RESET_CODE, RESET_COPY, RESET_CARD].contains(&reset_type) {
            return;
        }
        if reset_type == RESET_EVENT {
            let moved = RESET_TODECK | RESET_TOHAND | RESET_TOGRAVE | RESET_REMOVE | RESET_TEMP_REMOVE | RESET_OVERLAY | RESET_MSCHANGE;
            if id & moved != 0 {
                self.clear_relate_effect(card);
            }
            if id & (moved | RESET_LEAVE | RESET_TOFIELD) != 0 {
                let card = &mut self.cards[card];
                card.attack_announce_count = 0;
                card.announce_count = 0;
                card.attacked_count = 0;
                card.attack_all_target = 1;
            }
            if id & (moved | RESET_LEAVE | RESET_TOFIELD | RESET_TURN_SET) != 0 {
                self.reset_effect_count(card);
            }
            self.reset_counters(card, id);
            self.card_relations
                .retain(|&(owner, _, reset)| owner != card || reset & 0xffff_0000 & id == 0);
        }
        for effect in self.card_effects(card) {
            if self.effect_reset(effect, id, reset_type) {
                self.card_remove_effect(card, effect);
            }
        }
    }

    fn reset_effect_count(&mut self, card: CardId) {
        for effect in self.card_effects(card) {
            self.recharge(effect);
        }
    }

    pub(crate) fn create_relation(&mut self, card: CardId, effect: EffectId, chain_id: u16) {
        let relation = Relation { card, effect, chain_id };
        if !self.relations.contains(&relation) {
            self.relations.push(relation);
        }
    }

    pub(crate) fn is_has_relation(&self, card: CardId, effect: EffectId, chain_id: u16) -> bool {
        self.relations.contains(&Relation { card, effect, chain_id })
    }

    pub(crate) fn release_relation(&mut self, card: CardId, effect: EffectId, chain_id: u16) {
        if let Some(at) = self
            .relations
            .iter()
            .position(|&relation| relation == Relation { card, effect, chain_id })
        {
            self.relations.remove(at);
        }
    }

    // card::effect_target_cards: ordered by card id, as the core's card_set.
    pub(crate) fn card_targets(&self, card: CardId) -> CardSet {
        self.card_targets
            .iter()
            .filter(|&&(owner, _)| owner == card)
            .map(|&(_, target)| target)
            .collect()
    }

    fn target_effects_disable_related(&self, card: CardId) -> bool {
        self.field_effects
            .registry
            .all(Container::Target, Some(card))
            .any(|(_, effect)| self.effects[effect].is_disable_related())
    }

    pub(crate) fn add_card_target(&mut self, card: CardId, target: CardId) {
        if !self.card_targets.contains(&(card, target)) {
            self.card_targets.push((card, target));
        }
        if self.target_effects_disable_related(card) {
            self.add_to_disable_check_list(target);
        }
        let (from, to) = (self.cards.get_info_location(card), self.cards.get_info_location(target));
        self.trace.message(MSG_CARD_TARGET).loc(from).loc(to);
    }

    pub(crate) fn clear_card_target(&mut self, card: CardId) {
        let links: Vec<(CardId, CardId)> = self
            .card_targets
            .iter()
            .copied()
            .filter(|&(owner, target)| owner == card || target == card)
            .collect();
        if links.is_empty() {
            return;
        }
        self.card_targets.retain(|&(owner, target)| owner != card && target != card);
        let owners = links.iter().filter(|&&(_, target)| target == card).map(|&(owner, _)| owner);
        for owner in owners.collect::<CardSet>() {
            if self.target_effects_disable_related(owner) {
                self.add_to_disable_check_list(card);
            }
        }
        let targets = links.iter().filter(|&&(owner, _)| owner == card).map(|&(_, target)| target);
        for target in targets.collect::<CardSet>() {
            if self.target_effects_disable_related(card) {
                self.add_to_disable_check_list(target);
            }
            let related = self
                .field_effects
                .registry
                .all(Container::Single, Some(target))
                .map(|(_, effect)| effect);
            let related: List<EffectId, 8> = related
                .filter(|&effect| self.effects[effect].owner == card && self.effects[effect].is_flag(EFFECT_FLAG_OWNER_RELATE))
                .collect();
            for effect in related {
                self.card_remove_effect(target, effect);
            }
        }
    }

    pub(crate) fn clear_relate_effect(&mut self, card: CardId) {
        self.relations.retain(|relation| relation.card != card);
    }

    pub(crate) fn is_has_relation_effect(&self, card: CardId, effect: EffectId) -> bool {
        self.relations
            .iter()
            .any(|relation| relation.card == card && relation.effect == effect)
    }

    // card::filter_effect: the card's own effects, those of its equips, targets and materials, then the field's.
    #[inline(always)]
    pub(crate) fn filter_effect(&self, card: CardId, code: u32, sort: bool) -> EffectSet {
        if !self.field_effects.registry.has_code(code) {
            return EffectSet::default();
        }
        self.filter_effect_lookup(card, code, sort)
    }

    fn filter_effect_lookup(&self, card: CardId, code: u32, sort: bool) -> EffectSet {
        let mut effects = EffectSet::default();
        self.filter_effect_into(card, code, &mut effects);
        if sort && effects.len() > 1 {
            effects.sort_by_key(|&effect| self.effects.id(effect));
        }
        effects
    }

    fn filter_effect_into(&self, card: CardId, code: u32, effects: &mut EffectSet) {
        let registry = &self.field_effects.registry;
        effects.extend(registry.equal_range(Container::Single, Some(card), code).filter(|&effect| {
            self.is_available(effect)
                && (!self.effects[effect].is_flag(EFFECT_FLAG_SINGLE_RANGE) || self.is_affect_by_effect(card, Some(effect)))
        }));
        if !self.equip_cards.is_empty() {
            effects.extend(
                self.equip_effects(card, code)
                    .filter(|&effect| self.is_available(effect) && self.is_affect_by_effect(card, Some(effect))),
            );
        }
        effects.extend(registry.equal_range(Container::Aura, None, code).filter(|&effect| {
            !self.effects[effect].is_flag(EFFECT_FLAG_PLAYER_TARGET)
                && self.is_available(effect)
                && self.effect_is_target(effect, card)
                && self.is_affect_by_effect(card, Some(effect))
        }));
    }

    // The effects of `code` of the cards equipped to the card (equip_effect of its equiping_cards, by card id). Hot
    // lookups test `equip_cards` first, as nothing is equipped in most duels.
    fn equip_effects(&self, card: CardId, code: u32) -> impl Iterator<Item = EffectId> + '_ {
        self.equiping_cards(card)
            .into_iter()
            .flat_map(move |equip| self.field_effects.registry.equal_range(Container::Equip, Some(equip), code))
    }

    pub(crate) fn equiping_cards(&self, card: CardId) -> CardSet {
        self.equip_cards
            .iter()
            .filter(|&equip| self.cards[equip].equiping_target == Some(card))
            .collect()
    }

    fn equip_effects_disable_related(&self, card: CardId) -> bool {
        self.field_effects
            .registry
            .all(Container::Equip, Some(card))
            .any(|(_, effect)| self.effects[effect].is_disable_related())
    }

    // card::equip, which also writes MSG_EQUIP.
    pub(crate) fn card_equip(&mut self, card: CardId, target: CardId) {
        if self.cards[card].equiping_target.is_some() {
            return;
        }
        self.cards[card].equiping_target = Some(target);
        self.equip_cards.insert(card);
        if self.equip_effects_disable_related(card) {
            self.add_to_disable_check_list(target);
        }
        let (from, to) = (self.cards.get_info_location(card), self.cards.get_info_location(target));
        self.trace.message(MSG_EQUIP).loc(from).loc(to);
    }

    pub(crate) fn card_unequip(&mut self, card: CardId) {
        let Some(target) = self.cards[card].equiping_target else { return };
        if self.equip_effects_disable_related(card) {
            self.add_to_disable_check_list(target);
        }
        self.cards[card].equiping_target = None;
        self.equip_cards.remove(card);
    }

    // card::get_card_effect: like filter_effect unsorted, plus the card's own field effects whether available or not.
    pub(crate) fn get_card_effect(&self, card: CardId, code: u32) -> EffectSet {
        let mut effects = EffectSet::default();
        if !self.field_effects.registry.has_code(code) {
            return effects;
        }
        let registry = &self.field_effects.registry;
        effects.extend(registry.equal_range(Container::Single, Some(card), code).filter(|&effect| {
            self.is_available(effect)
                && (!self.effects[effect].is_flag(EFFECT_FLAG_SINGLE_RANGE) || self.is_affect_by_effect(card, Some(effect)))
        }));
        effects.extend(
            registry
                .equal_range(Container::Field, Some(card), code)
                .filter(|&effect| self.is_affect_by_effect(card, Some(effect))),
        );
        effects.extend(
            self.equip_effects(card, code)
                .filter(|&effect| self.is_available(effect) && self.is_affect_by_effect(card, Some(effect))),
        );
        effects.extend(registry.equal_range(Container::Aura, None, code).filter(|&effect| {
            !self.effects[effect].is_flag(EFFECT_FLAG_PLAYER_TARGET)
                && self.effect_is_target(effect, card)
                && self.is_available(effect)
                && self.is_affect_by_effect(card, Some(effect))
        }));
        effects
    }

    // card::filter_single_continuous_effect: the card's single effects regardless of their state.
    pub(crate) fn filter_single_continuous_effect(&self, card: CardId, code: u32) -> EffectSet {
        let mut effects: EffectSet = self
            .field_effects
            .registry
            .equal_range(Container::Single, Some(card), code)
            .collect();
        if !self.equip_cards.is_empty() {
            effects.extend(self.equip_effects(card, code));
        }
        effects.sort_by_key(|&effect| self.effects.id(effect));
        effects
    }

    // card::filter_immune_effect.
    pub(crate) fn filter_immune_effect(&self, card: CardId) -> EffectSet {
        let registry = &self.field_effects.registry;
        let mut effects: EffectSet = registry.equal_range(Container::Single, Some(card), EFFECT_IMMUNE_EFFECT).collect();
        if !self.equip_cards.is_empty() {
            effects.extend(self.equip_effects(card, EFFECT_IMMUNE_EFFECT));
        }
        effects.extend(
            registry
                .equal_range(Container::Aura, None, EFFECT_IMMUNE_EFFECT)
                .filter(|&effect| self.effect_is_target(effect, card)),
        );
        effects.sort_by_key(|&effect| self.effects.id(effect));
        effects
    }

    #[inline(always)]
    pub(crate) fn is_affected_by_effect(&self, card: CardId, code: u32) -> Option<EffectId> {
        if !self.field_effects.registry.has_code(code) {
            return None;
        }
        self.is_affected_by_effect_lookup(card, code)
    }

    fn is_affected_by_effect_lookup(&self, card: CardId, code: u32) -> Option<EffectId> {
        let registry = &self.field_effects.registry;
        registry
            .equal_range(Container::Single, Some(card), code)
            .find(|&effect| {
                self.is_available(effect)
                    && (!self.effects[effect].is_flag(EFFECT_FLAG_SINGLE_RANGE) || self.is_affect_by_effect(card, Some(effect)))
            })
            .or_else(|| {
                if self.equip_cards.is_empty() {
                    return None;
                }
                self.equip_effects(card, code)
                    .find(|&effect| self.is_available(effect) && self.is_affect_by_effect(card, Some(effect)))
            })
            .or_else(|| {
                registry.equal_range(Container::Aura, None, code).find(|&effect| {
                    !self.effects[effect].is_flag(EFFECT_FLAG_PLAYER_TARGET)
                        && self.effect_is_target(effect, card)
                        && self.is_available(effect)
                        && self.is_affect_by_effect(card, Some(effect))
                })
            })
    }

    // card::is_affected_by_effect(code, target): an effect that applies and whose value with the target is nonzero.
    #[inline(always)]
    pub(crate) fn is_affected_by_effect_target(&self, card: CardId, code: u32, target: CardId) -> Option<EffectId> {
        if !self.field_effects.registry.has_code(code) {
            return None;
        }
        self.is_affected_by_effect_target_lookup(card, code, target)
    }

    fn is_affected_by_effect_target_lookup(&self, card: CardId, code: u32, target: CardId) -> Option<EffectId> {
        let registry = &self.field_effects.registry;
        registry
            .equal_range(Container::Single, Some(card), code)
            .find(|&effect| {
                self.is_available(effect)
                    && (!self.effects[effect].is_flag(EFFECT_FLAG_SINGLE_RANGE) || self.is_affect_by_effect(card, Some(effect)))
                    && self.get_value(effect, target) != 0
            })
            .or_else(|| {
                if self.equip_cards.is_empty() {
                    return None;
                }
                // The core runs this loop a second time skipping field effects, which finds nothing new.
                self.equip_effects(card, code).find(|&effect| {
                    self.is_available(effect) && self.is_affect_by_effect(card, Some(effect)) && self.get_value(effect, target) != 0
                })
            })
            .or_else(|| {
                registry.equal_range(Container::Aura, None, code).find(|&effect| {
                    !self.effects[effect].is_flag(EFFECT_FLAG_PLAYER_TARGET)
                        && self.is_available(effect)
                        && self.effect_is_target(effect, card)
                        && self.is_affect_by_effect(card, Some(effect))
                        && self.get_value(effect, target) != 0
                })
            })
    }

    // Whether the card is affected by the effect: always when there is none, else not while summoning or immune.
    pub(crate) fn is_affect_by_effect(&self, card: CardId, effect: Option<EffectId>) -> bool {
        let Some(effect) = effect else { return true };
        let code = self.effects[effect].code;
        if self.cards[card].is_status(STATUS_SUMMONING) && code != EFFECT_CANNOT_DISABLE_SUMMON && code != EFFECT_CANNOT_DISABLE_SPSUMMON {
            return false;
        }
        if self.effects[effect].is_flag(EFFECT_FLAG_IGNORE_IMMUNE) {
            return true;
        }
        !self.is_immuned(effect, card)
    }

    // `sumtype` is 0 where the core passes none.
    pub(crate) fn check_cost_condition(&self, card: CardId, code: u32, playerid: u8, sumtype: u32) -> bool {
        let effects = self
            .filter_player_effect(playerid, code, false)
            .into_iter()
            .chain(self.filter_effect(card, code, true));
        for effect in effects {
            let peffect = &self.effects[effect];
            unported!(self, peffect.cost.map(|_| effect));
            if peffect
                .cost_condition
                .is_some_and(|cost| !cost(self, effect, card, playerid, sumtype))
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_summonable_card(&self, card: CardId) -> bool {
        let card_type = self.cards[card].data.card_type;
        let unsummonable = TYPE_RITUAL | TYPE_SPSUMMON | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK | TYPE_TOKEN | TYPE_TRAPMONSTER;
        card_type & TYPE_MONSTER != 0
            && card_type & unsummonable == 0
            && self.is_affected_by_effect(card, EFFECT_UNSUMMONABLE_CARD).is_none()
    }

    // Puts every summon procedure except the ordinary summon in the set (see is_can_be_summoned); the i32 is
    // TRUE/FALSE for the ordinary summon, or -1/-2 when a limiting procedure decides alone. Summon procedures come
    // with the cards that register them.
    pub(crate) fn filter_summon_procedure(
        &mut self,
        card: CardId,
        playerid: u8,
        ignore_count: bool,
        min_tribute: u8,
        zone: u32,
    ) -> (i32, EffectSet) {
        let check = |duel: &Self, effect| duel.check_summon_procedure(card, effect, playerid, ignore_count, min_tribute, zone);
        let limits = self.filter_effect(card, EFFECT_LIMIT_SUMMON_PROC, true);
        if !limits.is_empty() {
            let procedures: EffectSet = limits.into_iter().filter(|&effect| check(self, effect)).collect();
            return (if procedures.is_empty() { -2 } else { -1 }, procedures);
        }
        let procedures: EffectSet = self
            .filter_effect(card, EFFECT_SUMMON_PROC, true)
            .into_iter()
            .filter(|&effect| check(self, effect))
            .collect();
        if !self.is_player_can_summon(SUMMON_TYPE_NORMAL, playerid, card, playerid) {
            return (0, procedures);
        }
        if self.check_unique_onfield(card, playerid, LOCATION_MZONE).is_some() {
            return (0, procedures);
        }
        let rcount = self.get_summon_tribute_count(card);
        let mut min = (rcount & 0xffff) as u8;
        let mut max = ((rcount >> 16) & 0xffff) as u8;
        if !self.is_player_can_summon(SUMMON_TYPE_ADVANCE, playerid, card, playerid) {
            max = 0;
        }
        min = min.max(min_tribute);
        if max < min {
            return (0, procedures);
        }
        if !ignore_count
            && self.core.extra_summon[playerid] == 0
            && self.core.summon_count[playerid] >= self.get_summon_count_limit(playerid)
        {
            unported!(self, self.filter_effect(card, EFFECT_EXTRA_SUMMON_COUNT, true));
            return (0, procedures);
        }
        let controler = self.cards[card].current.controler;
        (
            i32::from(self.field_check_tribute(card, i32::from(min), i32::from(max), None, controler, zone, 0xff00ff)),
            procedures,
        )
    }

    // card::check_summon_procedure; the script checks min_tribute with Duel.CheckTribute.
    pub(crate) fn check_summon_procedure(
        &self,
        card: CardId,
        effect: EffectId,
        playerid: u8,
        ignore_count: bool,
        min_tribute: u8,
        zone: u32,
    ) -> bool {
        if !self.check_count_limit(effect, playerid) {
            return false;
        }
        let peffect = &self.effects[effect];
        let toplayer = if peffect.is_flag(EFFECT_FLAG_SPSUM_PARAM) && peffect.o_range != 0 {
            1 - playerid
        } else {
            playerid
        };
        if !self.is_player_can_summon(self.get_value(effect, card) as u32, playerid, card, toplayer)
            || self.check_unique_onfield(card, toplayer, LOCATION_MZONE).is_some()
        {
            return false;
        }
        if !ignore_count
            && self.core.extra_summon[playerid] == 0
            && self.core.summon_count[playerid] >= self.get_summon_count_limit(playerid)
        {
            unported!(self, self.filter_effect(card, EFFECT_EXTRA_SUMMON_COUNT, true));
            return false;
        }
        self.is_summonable(card, effect, min_tribute, zone, 0xff00ff)
    }

    // card::is_summonable: the procedure's condition (e, c, minc, zone, relzone, exeff) with exeff nil.
    pub(crate) fn is_summonable(&self, card: CardId, effect: EffectId, min_tribute: u8, zone: u32, releasable: u32) -> bool {
        let args = Args {
            e: effect,
            c: Some(card),
            min: u16::from(min_tribute),
            zone,
            releasable,
            ..Args::default()
        };
        self.check_procedure_condition_with(card, effect, &args)
    }

    // Same as filter_summon_procedure for setting.
    pub(crate) fn filter_set_procedure(
        &mut self,
        card: CardId,
        playerid: u8,
        ignore_count: bool,
        min_tribute: u8,
        zone: u32,
    ) -> (i32, EffectSet) {
        let check = |duel: &Self, effect| duel.check_set_procedure(card, effect, playerid, ignore_count, min_tribute, zone);
        let limits = self.filter_effect(card, EFFECT_LIMIT_SET_PROC, true);
        if !limits.is_empty() {
            let procedures: EffectSet = limits.into_iter().filter(|&effect| check(self, effect)).collect();
            return (if procedures.is_empty() { -2 } else { -1 }, procedures);
        }
        let procedures: EffectSet = self
            .filter_effect(card, EFFECT_SET_PROC, true)
            .into_iter()
            .filter(|&effect| check(self, effect))
            .collect();
        if !self.is_player_can_mset(SUMMON_TYPE_NORMAL, playerid, card, playerid) {
            return (0, procedures);
        }
        let rcount = self.get_set_tribute_count(card);
        let mut min = rcount & 0xffff;
        let mut max = (rcount >> 16) & 0xffff;
        if !self.is_player_can_mset(SUMMON_TYPE_ADVANCE, playerid, card, playerid) {
            max = 0;
        }
        min = min.max(i32::from(min_tribute));
        if max < min {
            return (0, procedures);
        }
        if !ignore_count
            && self.core.extra_summon[playerid] == 0
            && self.core.summon_count[playerid] >= self.get_summon_count_limit(playerid)
        {
            unported!(self, self.filter_effect(card, EFFECT_EXTRA_SET_COUNT, true));
            return (0, procedures);
        }
        let controler = self.cards[card].current.controler;
        (
            i32::from(self.field_check_tribute(card, min, max, None, controler, zone, 0xff00ff)),
            procedures,
        )
    }

    // card::check_set_procedure.
    pub(crate) fn check_set_procedure(
        &self,
        card: CardId,
        effect: EffectId,
        playerid: u8,
        ignore_count: bool,
        min_tribute: u8,
        zone: u32,
    ) -> bool {
        if !self.check_count_limit(effect, playerid) {
            return false;
        }
        let peffect = &self.effects[effect];
        let toplayer = if peffect.is_flag(EFFECT_FLAG_SPSUM_PARAM) && peffect.o_range != 0 {
            1 - playerid
        } else {
            playerid
        };
        if !self.is_player_can_mset(self.get_value(effect, card) as u32, playerid, card, toplayer) {
            return false;
        }
        if !ignore_count
            && self.core.extra_summon[playerid] == 0
            && self.core.summon_count[playerid] >= self.get_summon_count_limit(playerid)
        {
            unported!(self, self.filter_effect(card, EFFECT_EXTRA_SET_COUNT, true));
            return false;
        }
        self.is_summonable(card, effect, min_tribute, zone, 0xff00ff)
    }

    // Whether playerid can normal summon the card; `procedure` restricts to one summon procedure (c80921533).
    pub(crate) fn is_can_be_summoned(
        &mut self,
        card: CardId,
        playerid: u8,
        ignore_count: bool,
        procedure: Option<EffectId>,
        min_tribute: u8,
        zone: u32,
    ) -> bool {
        if !self.is_summonable_card(card) {
            return false;
        }
        self.mark_summon_self(card);
        if !ignore_count
            && (self.core.extra_summon[playerid] != 0 || self.is_affected_by_effect(card, EFFECT_EXTRA_SUMMON_COUNT).is_none())
            && self.core.summon_count[playerid] >= self.get_summon_count_limit(playerid)
        {
            return false;
        }
        if self.cards[card].is_status(STATUS_FORBIDDEN) || !self.check_cost_condition(card, EFFECT_SUMMON_COST, playerid, 0) {
            return false;
        }
        if self.cards[card].current.location == LOCATION_MZONE {
            return !(self.cards[card].is_position(POS_FACEDOWN)
                || self.is_affected_by_effect(card, EFFECT_GEMINI_SUMMONABLE).is_none()
                || self.is_affected_by_effect(card, EFFECT_GEMINI_STATUS).is_some()
                || !self.is_player_can_summon(SUMMON_TYPE_GEMINI, playerid, card, playerid)
                || self.is_affected_by_effect(card, EFFECT_CANNOT_SUMMON).is_some());
        }
        if self.is_affected_by_effect(card, EFFECT_CANNOT_SUMMON).is_some() {
            return false;
        }
        let (result, procedures) = self.filter_summon_procedure(card, playerid, ignore_count, min_tribute, zone);
        unported!(self, procedure);
        !procedures.is_empty() || (result != 0 && result != -2)
    }

    // The reason effect's status gets EFFECT_STATUS_SUMMON_SELF when it summons its own handler.
    fn mark_summon_self(&self, card: CardId) {
        if let Some(effect) = self.core.reason_effect.get()
            && self.get_handler(effect) == card
        {
            self.effects
                .set_status(effect, self.effects.status(effect) | EFFECT_STATUS_SUMMON_SELF);
        }
    }

    // The min/max number of tributes for an ordinary advance summon, packed as min + (max << 16).
    pub(crate) fn get_summon_tribute_count(&self, card: CardId) -> i32 {
        let level = self.get_level(card);
        let (min, max) = match level {
            ..5 => return 0,
            5..7 => (1, 1),
            _ => (2, 2),
        };
        unported!(self, self.filter_effect(card, EFFECT_DECREASE_TRIBUTE, true));
        min + (max << 16)
    }

    pub(crate) fn get_set_tribute_count(&self, card: CardId) -> i32 {
        let level = self.get_level(card);
        let (min, max) = match level {
            ..5 => return 0,
            5..7 => (1, 1),
            _ => (2, 2),
        };
        unported!(self, self.filter_effect(card, EFFECT_DECREASE_TRIBUTE_SET, true));
        min + (max << 16)
    }

    pub(crate) fn is_setable_mzone(
        &mut self,
        card: CardId,
        playerid: u8,
        ignore_count: bool,
        procedure: Option<EffectId>,
        min_tribute: u8,
        zone: u32,
    ) -> bool {
        if !self.is_summonable_card(card) {
            return false;
        }
        self.mark_summon_self(card);
        if self.cards[card].current.location == LOCATION_MZONE
            || self.cards[card].is_status(STATUS_FORBIDDEN)
            || self.is_affected_by_effect(card, EFFECT_CANNOT_MSET).is_some()
        {
            return false;
        }
        if !ignore_count
            && (self.core.extra_summon[playerid] != 0 || self.is_affected_by_effect(card, EFFECT_EXTRA_SET_COUNT).is_none())
            && self.core.summon_count[playerid] >= self.get_summon_count_limit(playerid)
        {
            return false;
        }
        if !self.check_cost_condition(card, EFFECT_MSET_COST, playerid, 0) {
            return false;
        }
        let (result, procedures) = self.filter_set_procedure(card, playerid, ignore_count, min_tribute, zone);
        unported!(self, procedure);
        !procedures.is_empty() || (result != 0 && result != -2)
    }

    pub(crate) fn is_setable_szone(&self, card: CardId, playerid: u8, ignore_fd: bool) -> bool {
        let (card_type, controler) = (self.cards[card].data.card_type, self.cards[card].current.controler);
        if card_type & TYPE_FIELD == 0
            && !ignore_fd
            && self
                .get_useable_count(Some(card), playerid, LOCATION_SZONE, controler, LOCATION_REASON_TOFIELD, 0xff)
                .0
                <= 0
        {
            return false;
        }
        if card_type & TYPE_MONSTER != 0 && self.is_affected_by_effect(card, EFFECT_MONSTER_SSET).is_none() {
            return false;
        }
        !self.cards[card].is_status(STATUS_FORBIDDEN)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_SSET).is_none()
            && self.is_player_can_sset(playerid, card)
            && self.check_cost_condition(card, EFFECT_SSET_COST, playerid, 0)
    }

    // card::filter_spsummon_procedure: the card's EFFECT_SPSUMMON_PROC effects it can be summoned with now.
    pub(crate) fn filter_spsummon_procedure(&self, card: CardId, playerid: u8, summon_type: u32, procedure: Procedure) -> EffectSet {
        let mut procedures = EffectSet::default();
        let effects: EffectSet = self
            .field_effects
            .registry
            .equal_range(Container::Field, Some(card), EFFECT_SPSUMMON_PROC)
            .collect();
        for effect in effects {
            let peffect = &self.effects[effect];
            let (topos, toplayer) = if peffect.is_flag(EFFECT_FLAG_SPSUM_PARAM) {
                (peffect.s_range as u8, if peffect.o_range == 0 { playerid } else { 1 - playerid })
            } else {
                (POS_FACEUP as u8, playerid)
            };
            if !self.is_available(effect) || !self.check_count_limit(effect, playerid) {
                continue;
            }
            if !self.is_spsummonable(card, effect, procedure)
                || (u32::from(topos) & POS_FACEDOWN == 0 && self.check_unique_onfield(card, toplayer, LOCATION_MZONE).is_some())
            {
                continue;
            }
            let summon_effect = self.core.reason_effect.get().unwrap_or(effect);
            let values = self.get_values(effect, card);
            let sumtype = values.first().map_or(0, |&value| value as u32);
            let zone = values.get(1).map_or(0xff, |&value| value as u32);
            let ignore_zone = values.get(2).is_some_and(|&value| value != 0);
            if zone != 0xff
                && !ignore_zone
                && self
                    .get_useable_count(Some(card), toplayer, LOCATION_MZONE, playerid, LOCATION_REASON_TOFIELD, zone)
                    .0
                    <= 0
            {
                continue;
            }
            if (summon_type != 0 && summon_type != sumtype)
                || !self.is_player_can_spsummon(Some(summon_effect), sumtype, topos, playerid, toplayer, card)
            {
                continue;
            }
            procedures.push(effect);
        }
        procedures
    }

    // card::filter_spsummon_procedure_g: the card's EFFECT_SPSUMMON_PROC_G effects whose condition holds.
    pub(crate) fn filter_spsummon_procedure_g(&self, card: CardId, playerid: u8) -> EffectSet {
        let effects: EffectSet = self
            .field_effects
            .registry
            .equal_range(Container::Field, Some(card), EFFECT_SPSUMMON_PROC_G)
            .collect();
        let mut procedures = EffectSet::default();
        for effect in effects {
            if !self.is_available(effect) || !self.check_count_limit(effect, playerid) {
                continue;
            }
            if self.cards[card].current.controler != playerid && !self.effects[effect].is_flag(EFFECT_FLAG_BOTH_SIDE) {
                continue;
            }
            if self.check_procedure_condition(card, effect) {
                procedures.push(effect);
            }
        }
        procedures
    }

    // card::is_spsummonable: the procedure's condition (e, c, must, materials, min, max), with the materials the core
    // keeps in must_use_mats and friends passed in `procedure`, which the procedure summon then carries.
    pub(crate) fn is_spsummonable(&self, card: CardId, effect: EffectId, procedure: Procedure) -> bool {
        let args = Args {
            e: effect,
            c: Some(card),
            must: procedure.must,
            materials: procedure.materials,
            min: procedure.min,
            max: procedure.max,
            ..Args::default()
        };
        self.check_procedure_condition_with(card, effect, &args)
    }

    // A summon procedure's condition (e, c), run with the procedure as the reason effect and the card's controller
    // as the reason player.
    pub(crate) fn check_procedure_condition(&self, card: CardId, effect: EffectId) -> bool {
        let args = Args {
            e: effect,
            c: Some(card),
            ..Args::default()
        };
        self.check_procedure_condition_with(card, effect, &args)
    }

    fn check_procedure_condition_with(&self, card: CardId, effect: EffectId, args: &Args) -> bool {
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.core.reason_effect.set(Some(effect));
        self.core.reason_player.set(self.cards[card].current.controler);
        let result = self.effects[effect]
            .condition
            .is_none_or(|condition| self.check_condition(condition, args));
        self.core.reason_effect.set(reason_effect);
        self.core.reason_player.set(reason_player);
        result
    }

    pub(crate) fn is_special_summonable(&self, card: CardId, playerid: u8, summon_type: u32, procedure: Procedure) -> bool {
        if self.cards[card].data.card_type & TYPE_MONSTER == 0
            || self.is_affected_by_effect(card, EFFECT_CANNOT_SPECIAL_SUMMON).is_some()
            || self.cards[card].is_status(STATUS_FORBIDDEN)
            || !self.check_cost_condition(card, EFFECT_SPSUMMON_COST, playerid, summon_type)
        {
            return false;
        }
        !self.filter_spsummon_procedure(card, playerid, summon_type, procedure).is_empty()
    }

    pub(crate) fn is_can_be_special_summoned(
        &self,
        card: CardId,
        reason_effect: EffectId,
        sumtype: u32,
        sumpos: u8,
        sumplayer: u8,
        toplayer: u8,
        nocheck: bool,
        nolimit: bool,
        zone: u32,
    ) -> bool {
        if self.get_handler(reason_effect) == card {
            self.effects
                .set_status(reason_effect, self.effects.status(reason_effect) | EFFECT_STATUS_SUMMON_SELF);
        }
        let target = &self.cards[card];
        let (location, data_type) = (target.current.location, target.data.card_type);
        if location == LOCATION_MZONE {
            return false;
        }
        if !nolimit && !target.is_status(STATUS_PROC_COMPLETE) && self.is_affected_by_effect(card, EFFECT_REVIVE_LIMIT).is_some() {
            if location & (LOCATION_GRAVE | LOCATION_REMOVED | LOCATION_SZONE) != 0
                || (!nocheck && location & (LOCATION_DECK | LOCATION_HAND) != 0)
            {
                return false;
            }
            if (data_type & TYPE_PENDULUM != 0 && location == LOCATION_EXTRA && target.current.position & POS_FACEUP != 0)
                || location == LOCATION_OVERLAY
            {
                return false;
            }
        }
        if data_type & TYPE_PENDULUM != 0
            && location == LOCATION_EXTRA
            && target.current.position & POS_FACEUP != 0
            && matches!(sumtype, SUMMON_TYPE_FUSION | SUMMON_TYPE_SYNCHRO | SUMMON_TYPE_XYZ)
        {
            return false;
        }
        let mut sumpos = sumpos;
        if u32::from(sumpos) & POS_FACEDOWN != 0 && self.is_player_affected_by_effect(sumplayer, EFFECT_DEVINE_LIGHT).is_some() {
            sumpos = (sumpos & POS_FACEUP as u8) | ((sumpos & POS_FACEDOWN as u8) >> 1);
        }
        if u32::from(sumpos) & POS_FACEUP != 0 && self.check_unique_onfield(card, toplayer, LOCATION_MZONE).is_some() {
            sumpos &= !(POS_FACEUP as u8);
            if sumpos == 0 {
                return false;
            }
        }
        let sumtype = sumtype | SUMMON_TYPE_SPECIAL;
        if sumplayer < 2 && !self.is_player_can_spsummon(Some(reason_effect), sumtype, sumpos, sumplayer, toplayer, card) {
            return false;
        }
        if self.is_affected_by_effect(card, EFFECT_CANNOT_SPECIAL_SUMMON).is_some() || self.cards[card].is_status(STATUS_FORBIDDEN) {
            return false;
        }
        if zone != 0xff
            && self
                .get_useable_count(Some(card), toplayer, LOCATION_MZONE, sumplayer, LOCATION_REASON_TOFIELD, zone)
                .0
                <= 0
        {
            return false;
        }
        if !self.check_cost_condition(card, EFFECT_SPSUMMON_COST, sumplayer, sumtype) {
            return false;
        }
        if !nocheck {
            return data_type & TYPE_MONSTER != 0
                && self.is_spsummon_condition_met(card, Some(reason_effect), sumplayer, sumtype, sumpos, toplayer);
        }
        true
    }

    pub(crate) fn is_releasable_by_summon(&self, card: CardId, playerid: u8, target: CardId) -> bool {
        let current = &self.cards[card];
        !current.is_status(STATUS_SUMMONING)
            && current.overlay_target.is_none()
            && current.current.location & (LOCATION_GRAVE | LOCATION_REMOVED) == 0
            && self.is_player_can_release(playerid, card, REASON_SUMMON)
            && self.is_affected_by_effect_target(card, EFFECT_UNRELEASABLE_SUM, target).is_none()
            && self.is_affected_by_effect_target(target, EFFECT_TRIBUTE_LIMIT, card).is_none()
    }

    pub(crate) fn is_releasable_by_nonsummon(&self, card: CardId, playerid: u8, reason: u32) -> bool {
        let target = &self.cards[card];
        !target.is_status(STATUS_SUMMONING)
            && target.overlay_target.is_none()
            && target.current.location & (LOCATION_GRAVE | LOCATION_REMOVED) == 0
            && !(target.current.location == LOCATION_HAND && target.data.card_type & (TYPE_SPELL | TYPE_TRAP) != 0)
            && self.is_player_can_release(playerid, card, reason)
            && self.is_affected_by_effect(card, EFFECT_UNRELEASABLE_NONSUM).is_none()
    }

    // EFFECT_UNRELEASABLE_EFFECT comes with the cards that register it.
    pub(crate) fn is_releasable_by_effect(&self, card: CardId, effect: Option<EffectId>) -> bool {
        if effect.is_none() {
            return true;
        }
        let limits = self.filter_effect(card, EFFECT_UNRELEASABLE_EFFECT, true);
        unported!(self, limits.first());
        limits.is_empty()
    }

    fn is_summoned_this_turn_by_controler(&self, card: CardId) -> bool {
        let target = &self.cards[card];
        (target.is_status(STATUS_SUMMON_TURN) || target.is_status(STATUS_FLIP_SUMMON_TURN) || target.is_status(STATUS_SPSUMMON_TURN))
            && (target.summon.player == target.current.controler || !self.is_flag(DUEL_CAN_REPOS_IF_NON_SUMPLAYER))
    }

    pub(crate) fn is_capable_change_position(&self, card: CardId, playerid: u8) -> bool {
        let target = &self.cards[card];
        !target.is_status(STATUS_FORM_CHANGED)
            && !self.is_summoned_this_turn_by_controler(card)
            && !(target.data.card_type & TYPE_LINK != 0 && target.data.card_type & TYPE_MONSTER != 0)
            && target.announce_count == 0
            && !target.is_status(STATUS_FORBIDDEN)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_CHANGE_POSITION).is_none()
            && self.is_player_affected_by_effect(playerid, EFFECT_CANNOT_CHANGE_POSITION).is_none()
    }

    pub(crate) fn is_capable_turn_set(&self, card: CardId, playerid: u8) -> bool {
        let target = &self.cards[card];
        target.data.card_type & TYPE_TOKEN == 0
            && !(target.data.card_type & TYPE_LINK != 0 && target.data.card_type & TYPE_MONSTER != 0)
            && !target.is_position(POS_FACEDOWN)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_TURN_SET).is_none()
            && self.is_player_affected_by_effect(playerid, EFFECT_CANNOT_TURN_SET).is_none()
    }

    pub(crate) fn is_capable_attack(&self, card: CardId) -> bool {
        let target = &self.cards[card];
        let unstoppable = || self.is_affected_by_effect(card, EFFECT_UNSTOPPABLE_ATTACK).is_some();
        (target.is_position(POS_FACEUP_ATTACK)
            || (target.is_position(POS_FACEUP_DEFENSE) && self.is_affected_by_effect(card, EFFECT_DEFENSE_ATTACK).is_some()))
            && self.is_affected_by_effect(card, EFFECT_FORBIDDEN).is_none()
            && (self.is_affected_by_effect(card, EFFECT_CANNOT_ATTACK).is_none() || unstoppable())
            && (self.is_affected_by_effect(card, EFFECT_ATTACK_DISABLED).is_none() || unstoppable())
            && self.is_player_affected_by_effect(self.infos.turn_player, EFFECT_SKIP_BP).is_none()
    }

    pub(crate) fn is_capable_attack_announce(&self, card: CardId, playerid: u8) -> bool {
        self.is_capable_attack(card)
            && (self.is_affected_by_effect(card, EFFECT_CANNOT_ATTACK_ANNOUNCE).is_none()
                || self.is_affected_by_effect(card, EFFECT_UNSTOPPABLE_ATTACK).is_some())
            && self.check_cost_condition(card, EFFECT_ATTACK_COST, playerid, 0)
    }

    pub(crate) fn is_capable_be_effect_target(&self, card: CardId, effect: EffectId, playerid: u8) -> bool {
        let target = &self.cards[card];
        if target.is_status(STATUS_SUMMONING) || target.is_status(STATUS_BATTLE_DESTROYED) {
            return false;
        }
        if target.current.location & (LOCATION_DECK | LOCATION_EXTRA | LOCATION_HAND) != 0 {
            return false;
        }
        if target.data.card_type & TYPE_TOKEN != 0 && target.current.location & LOCATION_ONFIELD == 0 {
            return false;
        }
        for limit in self.filter_effect(card, EFFECT_CANNOT_BE_EFFECT_TARGET, true) {
            if self.check_value_activate(limit, effect, playerid) {
                return false;
            }
        }
        unported!(
            self,
            self.filter_effect(self.get_handler(effect), EFFECT_CANNOT_SELECT_EFFECT_TARGET, true)
        );
        true
    }

    pub(crate) fn is_destructable(&self, card: CardId) -> bool {
        let target = &self.cards[card];
        target.overlay_target.is_none() && target.current.location & (LOCATION_GRAVE | LOCATION_REMOVED) == 0
    }

    // Values are functions of (e, re, rp), which ignore the card the core also passes.
    pub(crate) fn check_indestructable_by_effect(&self, card: CardId, effect: Option<EffectId>, playerid: u8) -> Option<EffectId> {
        let effect = effect?;
        self.filter_effect(card, EFFECT_INDESTRUCTABLE_EFFECT, true)
            .into_iter()
            .find(|&indestructable| match self.effects[indestructable].value {
                Value::Int(value) => value != 0,
                Value::EffectPlayer(value) => value(self, indestructable, effect, playerid),
                _ => {
                    self.reach_unported(concat!(file!(), ":", line!()));
                    false
                }
            })
    }

    // check_value_condition of every EFFECT_SPSUMMON_CONDITION with (se, sp, st, pos, tp).
    pub(crate) fn is_spsummon_condition_met(&self, card: CardId, se: Option<EffectId>, sp: u8, st: u32, pos: u8, tp: u8) -> bool {
        self.filter_effect(card, EFFECT_SPSUMMON_CONDITION, true)
            .into_iter()
            .all(|condition| match self.effects[condition].value {
                Value::Int(value) => value != 0,
                Value::SpSummonCondition(value) => value(self, condition, se, sp, st, pos, tp),
                _ => {
                    self.reach_unported(concat!(file!(), ":", line!()));
                    true
                }
            })
    }

    pub(crate) fn is_can_be_flip_summoned(&self, card: CardId, playerid: u8) -> bool {
        let target = &self.cards[card];
        !target.is_status(STATUS_FORM_CHANGED)
            && !self.is_summoned_this_turn_by_controler(card)
            && target.announce_count == 0
            && target.current.location == LOCATION_MZONE
            && target.is_position(POS_FACEDOWN)
            && self.check_unique_onfield(card, playerid, LOCATION_MZONE).is_none()
            && self.is_player_can_flipsummon(playerid, card)
            && !target.is_status(STATUS_FORBIDDEN)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_FLIP_SUMMON).is_none()
            && self.is_affected_by_effect(card, EFFECT_CANNOT_CHANGE_POSITION).is_none()
            && self.check_cost_condition(card, EFFECT_FLIPSUMMON_COST, playerid, 0)
    }

    pub(crate) fn set_material(&mut self, card: CardId, materials: CardSet) {
        self.set_material_cards(card, materials);
        for material in materials.iter() {
            self.cards[material].current.reason_card = Some(card);
        }
        for effect in self.filter_effect(card, EFFECT_MATERIAL_CHECK, true) {
            match self.effects[effect].value {
                Value::MaterialCheck(value) => value(self, effect, card),
                Value::Int(_) => {}
                _ => debug_assert!(false, "a material check read with the wrong parameters"),
            }
        }
    }

    // card::material_cards, kept per duel for the few cards that have materials.
    pub(crate) fn material_cards(&self, card: CardId) -> CardSet {
        self.material_cards
            .iter()
            .find(|(other, _)| *other == card)
            .map_or_else(CardSet::default, |&(_, materials)| materials)
    }

    pub(crate) fn set_material_cards(&mut self, card: CardId, materials: CardSet) {
        self.material_cards.retain(|(other, _)| *other != card);
        if !materials.is_empty() {
            self.material_cards.push((card, materials));
        }
    }

    // card::xyz_materials as a group, ordered by card id like the card_sets the core collects them into.
    pub(crate) fn xyz_materials(&self, card: CardId) -> CardSet {
        self.overlay_cards
            .iter()
            .filter(|&material| self.cards[material].overlay_target == Some(card))
            .collect()
    }

    // card::xyz_add. No material has EFFECT_TYPE_XMATERIAL field effects to apply, since card_add_effect does not take
    // them yet.
    pub(crate) fn xyz_add(&mut self, card: CardId, material: CardId) {
        if self.cards[material].current.location != 0 {
            return;
        }
        let sequence = self.xyz_materials(card).len() as u32;
        let mat = &mut self.cards[material];
        mat.overlay_target = Some(card);
        mat.pre_overlay_target = None;
        mat.current.controler = PLAYER_NONE;
        mat.current.location = LOCATION_OVERLAY;
        mat.current.sequence = sequence;
        self.overlay_cards.insert(material);
    }

    pub(crate) fn xyz_remove(&mut self, card: CardId, material: CardId) {
        if self.cards[material].overlay_target != Some(card) {
            return;
        }
        if !self.core.current_chain.is_empty() {
            self.core.just_sent_cards.insert(material);
        }
        self.overlay_cards.remove(material);
        let mat = &mut self.cards[material];
        let current = mat.current;
        mat.previous.controler = current.controler;
        mat.previous.location = current.location;
        mat.previous.sequence = current.sequence;
        mat.previous.pzone = current.pzone;
        mat.current.controler = PLAYER_NONE;
        mat.current.location = 0;
        mat.current.sequence = 0;
        mat.pre_overlay_target = mat.overlay_target.take();
        for other in self.xyz_materials(card) {
            let sequence = &mut self.cards[other].current.sequence;
            if *sequence > current.sequence {
                *sequence -= 1;
            }
        }
    }

    pub(crate) fn leave_field_redirect(&self, card: CardId) -> u32 {
        if self.cards[card].data.card_type & TYPE_TOKEN != 0 {
            return 0;
        }
        let mut redirects = 0;
        for effect in self.filter_effect(card, EFFECT_LEAVE_FIELD_REDIRECT, true) {
            let redirect = self.get_value(effect, card) as u32;
            let player = self.get_handler_player(effect);
            if (redirect & u32::from(LOCATION_HAND) != 0
                && self.is_affected_by_effect(card, EFFECT_CANNOT_TO_HAND).is_none()
                && self.is_player_can_send_to_hand(player, card))
                || (redirect & u32::from(LOCATION_DECK) != 0
                    && self.is_affected_by_effect(card, EFFECT_CANNOT_TO_DECK).is_none()
                    && self.is_player_can_send_to_deck(player, card))
                || (redirect & u32::from(LOCATION_REMOVED) != 0
                    && self.is_affected_by_effect(card, EFFECT_CANNOT_REMOVE).is_none()
                    && self.field_is_player_can_remove(player, card, REASON_EFFECT))
            {
                redirects |= redirect;
            }
        }
        if redirects & u32::from(LOCATION_REMOVED) != 0 {
            u32::from(LOCATION_REMOVED)
        } else if redirects & u32::from(LOCATION_DECK) != 0 {
            [LOCATION_DECKBOT, LOCATION_DECKSHF]
                .into_iter()
                .find(|&deck| redirects & deck == deck)
                .unwrap_or(u32::from(LOCATION_DECK))
        } else if redirects & u32::from(LOCATION_HAND) != 0 {
            u32::from(LOCATION_HAND)
        } else {
            0
        }
    }

    pub(crate) fn is_capable_send_to_grave(&self, card: CardId, playerid: u8, reason: u32) -> bool {
        self.is_affected_by_effect(card, EFFECT_CANNOT_TO_GRAVE).is_none() && self.is_player_can_send_to_grave(playerid, card, reason)
    }

    pub(crate) fn is_capable_send_to_hand(&self, card: CardId, playerid: u8) -> bool {
        let current = &self.cards[card];
        !current.is_status(STATUS_LEAVE_CONFIRMED)
            && !(current.current.location == LOCATION_EXTRA && self.is_extra_deck_monster(card))
            && self.is_affected_by_effect(card, EFFECT_CANNOT_TO_HAND).is_none()
            && !(self.is_extra_deck_monster(card) && !self.is_capable_send_to_deck(card, playerid))
            && self.is_player_can_send_to_hand(playerid, card)
    }

    pub(crate) fn is_capable_send_to_deck(&self, card: CardId, playerid: u8) -> bool {
        let current = &self.cards[card];
        !current.is_status(STATUS_LEAVE_CONFIRMED)
            && !(current.current.location == LOCATION_EXTRA && self.is_extra_deck_monster(card))
            && self.is_affected_by_effect(card, EFFECT_CANNOT_TO_DECK).is_none()
            && self.is_player_can_send_to_deck(playerid, card)
    }

    pub(crate) fn is_capable_send_to_extra(&self, card: CardId, playerid: u8) -> bool {
        (self.is_extra_deck_monster(card) || self.cards[card].data.card_type & TYPE_PENDULUM != 0)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_TO_DECK).is_none()
            && self.is_player_can_send_to_deck(playerid, card)
    }

    pub(crate) fn is_removeable(&self, card: CardId, playerid: u8, position: u8, reason: u32) -> bool {
        self.field_is_player_can_remove(playerid, card, reason)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_REMOVE).is_none()
            && !(self.cards[card].data.card_type & TYPE_TOKEN != 0 && u32::from(position) & POS_FACEDOWN != 0)
    }

    // Where a cost would send the card after the redirects that leaving the field and the destination apply (the
    // core sets sendto_param.location to `destination` meanwhile, which no ported redirect value reads).
    // Also whether a redirect applied.
    fn cost_destination(&self, card: CardId, destination: u8) -> (u32, bool) {
        let (mut destination, mut redirected) = (u32::from(destination), false);
        if self.cards[card].current.location & LOCATION_ONFIELD != 0 {
            let redirect = self.leave_field_redirect(card) & 0xffff;
            if redirect != 0 {
                (destination, redirected) = (redirect, true);
            }
        }
        let redirect = self.destination_redirect(card, destination as u8) & 0xffff;
        if redirect != 0 {
            (destination, redirected) = (redirect, true);
        }
        (destination, redirected)
    }

    pub(crate) fn is_removeable_as_cost(&self, card: CardId, playerid: u8, position: u8) -> bool {
        let target = &self.cards[card];
        if target.current.location == LOCATION_REMOVED
            || self.is_affected_by_effect(card, EFFECT_CANNOT_USE_AS_COST).is_some()
            || !self.is_removeable(card, playerid, position, REASON_COST)
        {
            return false;
        }
        let (destination, redirected) = self.cost_destination(card, LOCATION_REMOVED);
        destination == u32::from(LOCATION_REMOVED) && !(redirected && u32::from(position) & POS_FACEDOWN != 0)
    }

    pub(crate) fn is_capable_cost_to_grave(&self, card: CardId, playerid: u8) -> bool {
        let target = &self.cards[card];
        if target.data.card_type & TYPE_TOKEN != 0
            || (target.data.card_type & TYPE_PENDULUM != 0
                && target.current.location & LOCATION_ONFIELD != 0
                && self.is_capable_send_to_extra(card, playerid))
            || target.current.location == LOCATION_GRAVE
            || self.is_affected_by_effect(card, EFFECT_CANNOT_USE_AS_COST).is_some()
            || self.is_affected_by_effect(card, EFFECT_CANNOT_TO_GRAVE_AS_COST).is_some()
            || !self.is_capable_send_to_grave(card, playerid, REASON_COST)
        {
            return false;
        }
        self.cost_destination(card, LOCATION_GRAVE).0 == u32::from(LOCATION_GRAVE)
    }

    pub(crate) fn is_capable_cost_to_hand(&self, card: CardId, playerid: u8) -> bool {
        if self.cards[card].data.card_type & TYPE_TOKEN != 0
            || self.is_extra_deck_monster(card)
            || self.cards[card].current.location == LOCATION_HAND
            || self.is_affected_by_effect(card, EFFECT_CANNOT_USE_AS_COST).is_some()
            || !self.is_capable_send_to_hand(card, playerid)
        {
            return false;
        }
        self.cost_destination(card, LOCATION_HAND).0 == u32::from(LOCATION_HAND)
    }

    pub(crate) fn is_capable_cost_to_deck(&self, card: CardId, playerid: u8) -> bool {
        if self.cards[card].data.card_type & TYPE_TOKEN != 0
            || self.is_extra_deck_monster(card)
            || self.cards[card].current.location == LOCATION_DECK
            || self.is_affected_by_effect(card, EFFECT_CANNOT_USE_AS_COST).is_some()
            || !self.is_capable_send_to_deck(card, playerid)
        {
            return false;
        }
        self.cost_destination(card, LOCATION_DECK).0 == u32::from(LOCATION_DECK)
    }

    pub(crate) fn is_capable_cost_to_extra(&self, card: CardId, playerid: u8) -> bool {
        if !self.is_extra_deck_monster(card)
            || self.cards[card].current.location == LOCATION_EXTRA
            || self.is_affected_by_effect(card, EFFECT_CANNOT_USE_AS_COST).is_some()
            || !self.is_capable_send_to_deck(card, playerid)
        {
            return false;
        }
        self.cost_destination(card, LOCATION_EXTRA).0 == u32::from(LOCATION_EXTRA)
    }

    pub(crate) fn destination_redirect(&self, card: CardId, destination: u8) -> u32 {
        if self.cards[card].data.card_type & TYPE_TOKEN != 0 {
            return 0;
        }
        let code = match destination {
            LOCATION_HAND => EFFECT_TO_HAND_REDIRECT,
            LOCATION_DECK => EFFECT_TO_DECK_REDIRECT,
            LOCATION_GRAVE => EFFECT_TO_GRAVE_REDIRECT,
            LOCATION_REMOVED => EFFECT_REMOVE_REDIRECT,
            _ => return 0,
        };
        unported!(self, self.filter_effect(card, code, true));
        0
    }
}

// Materials of Synchro and Link Summons, and Link ratings.
impl Duel {
    pub(crate) fn is_not_tuner(&self, card: CardId, scard: Option<CardId>, playerid: u8) -> bool {
        if self.get_type_for(card, scard, SUMMON_TYPE_SYNCHRO.into(), playerid) & TYPE_TUNER == 0 {
            return true;
        }
        unported!(self, self.filter_effect(card, EFFECT_NONTUNER, true));
        false
    }

    pub(crate) fn get_synchro_level(&self, card: CardId) -> u32 {
        let data_type = self.cards[card].data.card_type;
        if data_type & TYPE_LINK != 0 && self.is_affected_by_effect(card, EFFECT_SYNCHRO_LEVEL).is_none() {
            return 0;
        }
        if (data_type & TYPE_XYZ != 0 || self.cards[card].status & STATUS_NO_LEVEL != 0)
            && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL).is_none()
            && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL_S).is_none()
        {
            return 0;
        }
        unported!(self, self.filter_effect(card, EFFECT_SYNCHRO_LEVEL, true));
        self.get_level(card)
    }

    pub(crate) fn get_ritual_level(&self, card: CardId, rcard: CardId) -> u32 {
        let data_type = self.cards[card].data.card_type;
        if data_type & TYPE_LINK != 0 {
            return 0;
        }
        if (data_type & TYPE_XYZ != 0 || self.cards[card].status & STATUS_NO_LEVEL != 0)
            && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL).is_none()
            && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL_S).is_none()
        {
            return 0;
        }
        match self.filter_effect(card, EFFECT_RITUAL_LEVEL, true).first() {
            Some(&effect) => self.get_value(effect, rcard) as u32,
            None => self.get_level(card),
        }
    }

    pub(crate) fn get_link(&self, card: CardId) -> u32 {
        let target = &self.cards[card];
        if target.data.card_type & TYPE_LINK == 0 || target.status & STATUS_NO_LEVEL != 0 {
            return 0;
        }
        if target.current.location & LOCATION_MZONE == 0 {
            return target.data.level;
        }
        unported!(
            self,
            self.filter_effects(card, &[EFFECT_UPDATE_LINK, EFFECT_CHANGE_LINK, EFFECT_CHANGE_LINK_FINAL])
        );
        let link = target.data.level;
        if link < 1 && self.get_type(card) & TYPE_MONSTER != 0 && self.is_affected_by_effect(card, EFFECT_ALLOW_NEGATIVE).is_none() {
            1
        } else {
            link
        }
    }

    // The values of EFFECT_CANNOT_BE_MATERIAL: whether one forbids the card as material of `scard`.
    fn is_forbidden_material(&self, card: CardId, scard: Option<CardId>, sumtype: u64, playerid: u8) -> bool {
        self.filter_effect(card, EFFECT_CANNOT_BE_MATERIAL, true)
            .into_iter()
            .any(|effect| match self.effects[effect].value {
                Value::Int(value) => value != 0,
                Value::Material(value) => value(self, effect, scard, sumtype, playerid),
                _ => {
                    debug_assert!(false, "a material limit read with the wrong parameters");
                    false
                }
            })
    }

    pub(crate) fn is_can_be_synchro_material(&self, card: CardId, scard: Option<CardId>, playerid: u8) -> bool {
        let target = &self.cards[card];
        let data_type = target.data.card_type;
        if Some(card) == scard
            || (data_type & TYPE_XYZ != 0
                && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL).is_none()
                && self.is_affected_by_effect(card, EFFECT_RANK_LEVEL_S).is_none())
            || (data_type & TYPE_LINK != 0 && self.is_affected_by_effect(card, EFFECT_SYNCHRO_LEVEL).is_none())
            || self.get_type_for(card, scard, SUMMON_TYPE_SYNCHRO.into(), playerid) & TYPE_MONSTER == 0
        {
            return false;
        }
        if let Some(scard) = scard
            && target.current.location == LOCATION_MZONE
            && target.current.controler != self.cards[scard].current.controler
            && self.is_affected_by_effect(card, EFFECT_SYNCHRO_MATERIAL).is_none()
        {
            return false;
        }
        if target.is_status(STATUS_FORBIDDEN) {
            return false;
        }
        unported!(self, self.filter_effect(card, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL, true));
        !self.is_forbidden_material(card, scard, SUMMON_TYPE_SYNCHRO.into(), playerid)
    }

    // EFFECT_EXTRA_RITUAL_MATERIAL, which lets a monster in the GY be a material, comes with the cards that register it.
    pub(crate) fn is_can_be_ritual_material(&self, card: CardId, scard: Option<CardId>, playerid: u8) -> bool {
        if Some(card) == scard || self.get_type(card) & TYPE_MONSTER == 0 {
            return false;
        }
        if self.is_forbidden_material(card, scard, SUMMON_TYPE_RITUAL.into(), playerid) {
            return false;
        }
        if self.cards[card].current.location == LOCATION_GRAVE {
            let extra = self.filter_effect(card, EFFECT_EXTRA_RITUAL_MATERIAL, true);
            unported!(self, extra.first());
            return false;
        }
        true
    }

    pub(crate) fn is_can_be_xyz_material(&self, card: CardId, scard: Option<CardId>, playerid: u8, reason: u32) -> bool {
        let target = &self.cards[card];
        if Some(card) == scard || target.data.card_type & TYPE_TOKEN != 0 {
            return false;
        }
        if (target.current.location & LOCATION_ONFIELD == 0 || reason & REASON_MATERIAL != 0) && target.is_status(STATUS_FORBIDDEN) {
            return false;
        }
        if reason & REASON_EFFECT != 0 {
            return target.current.controler == playerid || self.is_affected_by_effect(card, EFFECT_CANNOT_CHANGE_CONTROL).is_none();
        }
        if self.get_type_for(card, scard, SUMMON_TYPE_XYZ.into(), playerid) & TYPE_MONSTER == 0 {
            return false;
        }
        if reason & REASON_MATERIAL == 0 {
            return true;
        }
        let limits = self.filter_effect(card, EFFECT_CANNOT_BE_XYZ_MATERIAL, true);
        if limits
            .into_iter()
            .any(|effect| self.get_value(effect, scard.unwrap_or_default()) != 0)
        {
            return false;
        }
        !self.is_forbidden_material(card, scard, SUMMON_TYPE_XYZ.into(), playerid)
    }

    pub(crate) fn is_can_be_link_material(&self, card: CardId, scard: Option<CardId>, playerid: u8) -> bool {
        if Some(card) == scard
            || self.get_type_for(card, scard, SUMMON_TYPE_LINK.into(), playerid) & TYPE_MONSTER == 0
            || self.cards[card].is_status(STATUS_FORBIDDEN)
        {
            return false;
        }
        unported!(self, self.filter_effect(card, EFFECT_CANNOT_BE_LINK_MATERIAL, true));
        !self.is_forbidden_material(card, scard, SUMMON_TYPE_LINK.into(), playerid)
    }

    // EFFECT_CANNOT_BE_FUSION_MATERIAL values that are functions of (e, c, sumtype) and EFFECT_EXTRA_FUSION_MATERIAL
    // effects come with the cards that register them; proc_fusion_spell.lua's global one is not ported (see the scripts'
    // Fusion.ExtraGroup).
    pub(crate) fn is_can_be_fusion_material(&self, card: CardId, fcard: Option<CardId>, summon_type: u64, playerid: u8) -> bool {
        if Some(card) == fcard || self.cards[card].is_status(STATUS_FORBIDDEN) {
            return false;
        }
        let limited = self
            .filter_effect(card, EFFECT_CANNOT_BE_FUSION_MATERIAL, true)
            .into_iter()
            .any(|effect| match self.effects[effect].value {
                Value::Int(value) => value != 0,
                _ => {
                    self.reach_unported(concat!(file!(), ":", line!()));
                    false
                }
            });
        if limited || self.is_forbidden_material(card, fcard, summon_type, playerid) {
            return false;
        }
        if fcard.is_some() {
            unported!(self, self.filter_effect(card, EFFECT_EXTRA_FUSION_MATERIAL, true));
            return true;
        }
        let target = &self.cards[card];
        if target.current.location & LOCATION_ONFIELD == 0 && target.data.card_type & TYPE_MONSTER == 0 {
            let extra = self.is_affected_by_effect(card, EFFECT_EXTRA_FUSION_MATERIAL);
            unported!(self, extra);
            return extra.is_some();
        }
        true
    }

    // card::fusion_check and card::fusion_filter_valid: the EFFECT_FUSION_MATERIAL effects whose condition (e, g, gc,
    // chkf) holds, each run with the effect as the reason effect and its handler's player as the reason player. The core
    // walks the single effects from the code to the end of the container and skips other codes.
    pub(crate) fn fusion_materials_valid(
        &self,
        card: CardId,
        g: Option<CardSet>,
        gc: Option<CardSet>,
        chkf: u64,
    ) -> impl Iterator<Item = EffectId> + '_ {
        let registry = &self.field_effects.registry;
        registry
            .equal_range(Container::Single, Some(card), EFFECT_FUSION_MATERIAL)
            .filter(move |&effect| {
                let Some(condition) = self.effects[effect].condition else {
                    return false;
                };
                let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
                self.core.reason_effect.set(Some(effect));
                self.core.reason_player.set(self.get_handler_player(effect));
                let args = Args {
                    e: effect,
                    materials: g,
                    must: gc,
                    chkf,
                    ..Args::default()
                };
                let result = self.check_condition(condition, &args);
                self.core.reason_effect.set(reason_effect);
                self.core.reason_player.set(reason_player);
                result
            })
    }

    pub(crate) fn check_fusion_substitute(&self, card: CardId, fcard: CardId) -> bool {
        self.filter_effect(card, EFFECT_FUSION_SUBSTITUTE, true)
            .into_iter()
            .any(|effect| matches!(self.effects[effect].value, Value::Int(0)) || self.get_value(effect, fcard) != 0)
    }

    // card::get_summon_code as Card.IsSummonCode reads it; code effects with an operation, which apply only to such
    // checks, come with the cards that register them.
    pub(crate) fn is_summon_code(&self, card: CardId, codes: &[u32]) -> bool {
        for code in [EFFECT_ADD_CODE, EFFECT_REMOVE_CODE] {
            unported!(self, self.filter_effect(card, code, false));
        }
        let changes = self.filter_effect(card, EFFECT_CHANGE_CODE, true);
        unported!(self, changes.into_iter().filter(|&effect| self.effects[effect].operation.is_some()));
        let (code, another) = (self.get_code(card), self.get_another_code(card));
        codes.iter().any(|&other| other == code || (another != 0 && other == another))
    }
}

// card::counters, kept per duel sorted by card and counter type: the permanent count, then the count a negation
// removes.
impl Duel {
    fn counter_slot(&self, card: CardId, counter_type: u16) -> Result<usize, usize> {
        self.counters
            .binary_search_by_key(&(card, counter_type), |&(other, other_type, _)| (other, other_type))
    }

    pub(crate) fn get_counter(&self, card: CardId, counter_type: u16) -> u16 {
        self.counter_slot(card, counter_type)
            .ok()
            .and_then(|slot| self.counters.get(slot))
            .map_or(0, |&(_, _, [permanent, negatable])| permanent + negatable)
    }

    fn write_remove_counter(&mut self, card: CardId, counter_type: u16, count: u16) {
        let current = self.cards[card].current;
        self.trace
            .message(MSG_REMOVE_COUNTER)
            .u16(counter_type)
            .u8(current.controler)
            .u8(current.location)
            .u8(current.sequence as u8)
            .u16(count);
    }

    // The counters a reset clears: all of them when the card moves, the negatable ones when it is negated.
    fn reset_counters(&mut self, card: CardId, id: u32) {
        let moved = RESET_TODECK | RESET_TOHAND | RESET_TOGRAVE | RESET_REMOVE | RESET_TEMP_REMOVE | RESET_OVERLAY | RESET_MSCHANGE;
        if id & (moved | RESET_TOFIELD | RESET_TURN_SET) != 0 {
            self.counters.retain(|&(other, _, _)| other != card);
        }
        if id & RESET_DISABLE == 0 {
            return;
        }
        let mut slot = self.counters.partition_point(|&(other, _, _)| other < card);
        while let Some(&(other, counter_type, [permanent, negatable])) = self.counters.get(slot) {
            if other != card {
                break;
            }
            if negatable > 0 {
                self.write_remove_counter(card, counter_type, negatable);
                if permanent == 0 {
                    self.counters.remove(slot);
                    continue;
                }
                if let Some(counter) = self.counters.get_mut(slot) {
                    counter.2[1] = 0;
                }
            }
            slot += 1;
        }
    }

    // The counters of a type whose EFFECT_COUNTER_PERMIT effect left the card.
    fn remove_permitted_counters(&mut self, card: CardId, counter_type: u16) {
        if let Ok(slot) = self.counter_slot(card, counter_type) {
            let (_, _, [permanent, negatable]) = self.counters.remove(slot);
            self.write_remove_counter(card, counter_type, permanent + negatable);
        }
    }

    pub(crate) fn is_can_add_counter(
        &self,
        card: CardId,
        playerid: u8,
        counter_type: u16,
        count: u16,
        singly: bool,
        location: u32,
    ) -> bool {
        let target = &self.cards[card];
        if count > 0 {
            unported!(self, self.filter_player_effect(playerid, EFFECT_CANNOT_PLACE_COUNTER, true));
            if location == 0 && (target.current.location & LOCATION_ONFIELD == 0 || !target.is_position(POS_FACEUP)) {
                return false;
            }
            if counter_type & COUNTER_NEED_ENABLE != 0 && target.is_status(STATUS_DISABLED) {
                return false;
            }
        }
        let permitted = counter_type & COUNTER_WITHOUT_PERMIT != 0
            || self
                .filter_effect(card, EFFECT_COUNTER_PERMIT + u32::from(counter_type), true)
                .into_iter()
                .any(|effect| {
                    let range = self.get_value(effect, card) as u32;
                    if location != 0 {
                        location & range != 0
                    } else if target.current.location & LOCATION_ONFIELD != 0 {
                        unported!(self, matches!(self.effects[effect].target, Target::Card(_)).then_some(effect));
                        target.current.is_location(range as u16) && target.is_position(POS_FACEUP)
                    } else {
                        true
                    }
                });
        if !permitted {
            return false;
        }
        let counter_type = counter_type & !COUNTER_NEED_ENABLE;
        unported!(self, self.filter_effect(card, EFFECT_COUNTER_LIMIT + u32::from(counter_type), true));
        let _ = singly;
        true
    }

    pub(crate) fn add_counter(&mut self, card: CardId, playerid: u8, counter_type: u16, count: u16, singly: bool) -> bool {
        if !self.is_can_add_counter(card, playerid, counter_type, count, singly, 0) {
            return false;
        }
        let cttype = counter_type & !COUNTER_NEED_ENABLE;
        let slot = match self.counter_slot(card, cttype) {
            Ok(slot) => slot,
            Err(slot) => {
                self.counters.insert(slot, (card, cttype, [0, 0]));
                slot
            }
        };
        if let Some(counter) = self.counters.get_mut(slot) {
            let permanent = counter_type & COUNTER_WITHOUT_PERMIT != 0 && counter_type & COUNTER_NEED_ENABLE == 0;
            if let Some(slot) = counter.2.get_mut(usize::from(!permanent)) {
                *slot += count;
            }
        }
        let current = self.cards[card].current;
        self.trace
            .message(MSG_ADD_COUNTER)
            .u16(cttype)
            .u8(current.controler)
            .u8(current.location)
            .u8(current.sequence as u8)
            .u16(count);
        let reason_effect = self.core.reason_effect.get();
        self.field_raise_single_event(
            card,
            EVENT_ADD_COUNTER + u32::from(counter_type),
            reason_effect,
            REASON_EFFECT,
            playerid,
            playerid,
            u32::from(count),
        );
        self.process_single_event();
        true
    }

    pub(crate) fn card_remove_counter(&mut self, card: CardId, counter_type: u16, count: u16) -> bool {
        let Ok(slot) = self.counter_slot(card, counter_type) else {
            return false;
        };
        let Some(&(_, _, [permanent, negatable])) = self.counters.get(slot) else {
            return false;
        };
        if negatable <= count {
            let remains = count - negatable;
            if permanent <= remains {
                self.counters.remove(slot);
            } else if let Some(counter) = self.counters.get_mut(slot) {
                counter.2 = [permanent - remains, 0];
            }
        } else if let Some(counter) = self.counters.get_mut(slot) {
            counter.2[1] -= count;
        }
        self.write_remove_counter(card, counter_type, count);
        true
    }

    // field::get_field_counter: the counters on the monsters and spells of the players `self_side` and `oppo` name.
    pub(crate) fn get_field_counter(&self, playerid: u8, self_side: u8, oppo: u8, counter_type: u16) -> u32 {
        let mut count = 0;
        for (player, included) in [(playerid, self_side), (1 - playerid, oppo)] {
            if included == 0 {
                continue;
            }
            let info = &self.player[player];
            for card in info.list_mzone.into_iter().chain(info.list_szone).flatten() {
                count += u32::from(self.get_counter(card, counter_type));
            }
        }
        count
    }

    pub(crate) fn is_player_can_remove_counter(
        &self,
        playerid: u8,
        card: Option<CardId>,
        self_side: u8,
        oppo: u8,
        counter_type: u16,
        count: u16,
    ) -> bool {
        let enough = match card {
            Some(card) => self.get_counter(card, counter_type) >= count,
            None => self.get_field_counter(playerid, self_side, oppo, counter_type) >= u32::from(count),
        };
        if enough {
            return true;
        }
        unported!(
            self,
            self.field_effect_range(Container::Continuous, EFFECT_RCOUNTER_REPLACE + u32::from(counter_type))
        );
        false
    }

    // field::get_overlay_group: the materials of the cards of `group`, or of the monsters on the sides `self_side` and
    // `oppo` of `playerid`, leaving out monsters being summoned.
    pub(crate) fn field_overlay_group(&self, playerid: u8, self_side: u8, oppo: u8, group: Option<CardSet>) -> CardSet {
        let targets = group.unwrap_or_else(|| {
            [(playerid, self_side), (1 - playerid, oppo)]
                .into_iter()
                .filter(|&(_, included)| included != 0)
                .flat_map(|(player, _)| self.player[player].list_mzone.into_iter().flatten())
                .collect()
        });
        let holds = |card: CardId| targets.contains(card) && !self.cards[card].get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP);
        self.overlay_cards
            .iter()
            .filter(|&material| self.cards[material].overlay_target.is_some_and(holds))
            .collect()
    }

    // EFFECT_OVERLAY_REMOVE_REPLACE comes with the cards that register it.
    pub(crate) fn is_player_can_remove_overlay_card(
        &self,
        playerid: u8,
        group: Option<CardSet>,
        self_side: u8,
        oppo: u8,
        min: u16,
    ) -> bool {
        if self.field_overlay_group(playerid, self_side, oppo, group).len() >= usize::from(min) {
            return true;
        }
        unported!(self, self.field_effect_range(Container::Continuous, EFFECT_OVERLAY_REMOVE_REPLACE));
        false
    }
}

// Columns.
impl Duel {
    // card::get_column_zone: the zones of the card's column widened by `left` and `right`, the opponent's in the high
    // half, without the card's own zone.
    pub(crate) fn get_column_zone(&self, card: CardId, loc1: u32, left: i32, right: i32) -> u32 {
        let current = self.cards[card].current;
        let (loc2, own_sequence) = (u32::from(current.location), current.sequence as i32);
        if loc1 & u32::from(LOCATION_ONFIELD) == 0 || loc2 & u32::from(LOCATION_ONFIELD) == 0 || left < 0 || right < 0 {
            return 0;
        }
        if loc2 & u32::from(LOCATION_SZONE) != 0 && own_sequence >= 5 {
            return 0;
        }
        let mut sequence = own_sequence;
        if loc2 & u32::from(LOCATION_MZONE) != 0 && (sequence == 5 || sequence == 6) {
            sequence = if sequence == 5 { 1 } else { 3 };
        }
        let (first, last) = ((sequence - left).max(0), (sequence + right).min(4));
        let in_extra = current.location == LOCATION_MZONE && own_sequence >= 5;
        let mut zones = 0u32;
        for s in first..=last {
            if loc1 & u32::from(LOCATION_MZONE) != 0 {
                zones |= (1 << s) | (1 << (16 + (4 - s)));
                if self.is_flag(DUEL_EMZONE) {
                    if s == 1 && !(in_extra && own_sequence == 5) {
                        zones |= (1 << 5) | (1 << (16 + 6));
                    } else if s == 3 && !(in_extra && own_sequence == 6) {
                        zones |= (1 << 6) | (1 << (16 + 5));
                    }
                }
            }
            if loc1 & u32::from(LOCATION_SZONE) != 0 {
                zones |= (1 << (8 + s)) | (1 << (16 + 8 + (4 - s)));
            }
        }
        let own_bit = (1u32 << own_sequence) << if loc2 == u32::from(LOCATION_SZONE) { 8 } else { 0 };
        zones & !own_bit
    }

    // field::get_cards_in_zone for the whole field: bits 0-6 the monster zones, 8-15 the spell zones.
    fn get_cards_in_zone(&self, cards: &mut CardSet, zone: u32, playerid: u8) {
        let info = &self.player[playerid];
        for (bit, card) in info.list_mzone.iter().enumerate() {
            if zone & (1 << bit) != 0
                && let Some(card) = card
            {
                cards.insert(*card);
            }
        }
        for (bit, card) in info.list_szone.iter().enumerate() {
            if zone & (1 << (8 + bit)) != 0
                && let Some(card) = card
            {
                cards.insert(*card);
            }
        }
    }

    pub(crate) fn get_column_cards(&self, card: CardId, left: i32, right: i32) -> CardSet {
        let mut cards = CardSet::default();
        let current = self.cards[card].current;
        if current.location & LOCATION_ONFIELD == 0 {
            return cards;
        }
        let zone = self.get_column_zone(card, u32::from(LOCATION_ONFIELD), left, right);
        self.get_cards_in_zone(&mut cards, zone, current.controler);
        self.get_cards_in_zone(&mut cards, zone >> 16, 1 - current.controler);
        cards
    }
}
