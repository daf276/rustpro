// Port of effect.h/effect.cpp: effects are plain data in one per-duel arena, their behaviour static function pointers.
use std::cell::Cell;
use std::num::{NonZeroU16, NonZeroU64};
use std::ops::{Index, IndexMut};

use crate::card::{CardId, CardSet};
use crate::constants::*;
use crate::duel::Duel;
use crate::list::{List, Shared};
use crate::script::{Args, Script};

// The effect's index in the duel's arena. Slots of effects the core deletes are reused, as the core reuses memory.
// Stored plus one, so that an Option<EffectId> takes no more space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectId(NonZeroU16);

impl EffectId {
    fn new(index: u16) -> Self {
        Self(NonZeroU16::MIN.saturating_add(index))
    }

    pub fn index(self) -> usize {
        usize::from(self.0.get() - 1)
    }

    // The core's `deleted` object, which every Lua reference to a deleted effect then points at: no effect's index.
    const DELETED: Self = Self(NonZeroU16::MAX);
}

impl Default for EffectId {
    fn default() -> Self {
        Self::new(0)
    }
}

impl std::fmt::Debug for EffectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EffectId").field(&self.index()).finish()
    }
}

// effect_set: effects collected from containers, sorted by id where the core sorts them.
pub type EffectSet = List<EffectId, 8>;

pub type ConditionFn = fn(&Duel, &Args) -> bool;

// effect::condition. A condition whose Lua writes state that persists (its effect's label or label object, script
// state, flag effects) is `Mut`; the core reaches the conditions of actions only where the engine holds `&mut Duel`.
#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Check(ConditionFn),
    Mut(fn(&mut Duel, &Args) -> bool),
}

pub type CostFn = fn(&mut Duel, &Args, i32) -> Script<bool>;
pub type CostConditionFn = fn(&Duel, EffectId, CardId, u8, u32) -> bool;
pub type TargetFn = fn(&mut Duel, &Args, i32, Option<CardId>) -> Script<bool>;
pub type OperationFn = fn(&mut Duel, &Args) -> Script<()>;
// An operation whose return value the core reads: EFFECT_DISABLE_FIELD, the zones to disable.
pub type ReturningOperationFn = fn(&mut Duel, &Args) -> Script<u32>;
// A Duel.SetChainLimit function f(e, rp, tp), with the value its Lua closure captures (0 for none).
pub type ChainLimitFn = fn(&Duel, EffectId, u8, u8, i64) -> bool;

// chain_limit_t: the function and the player who set it.
#[derive(Debug, Clone, Copy)]
pub struct ChainLimit {
    pub function: ChainLimitFn,
    pub capture: i64,
    pub player: u8,
}

// The operation of EFFECT_SPSUMMON_PROC_G, which fills the group of cards to summon the core passes it.
pub type GroupOperationFn = fn(&mut Duel, &Args, &mut CardSet) -> Script<()>;

// effect::target holds a Lua function whose parameters depend on the effect: the activation target of actions, or
// the card filter (e, c) of field effects, which ignores the further parameters some codes pass.
#[derive(Debug, Clone, Copy, Default)]
pub enum Target {
    #[default]
    None,
    Action(TargetFn),
    Card(fn(&Duel, EffectId, CardId) -> bool),
    // The target (e, c, sump, sumtype, sumpos, targetp, se) of EFFECT_CANNOT_SPECIAL_SUMMON.
    SpSummon(SpSummonTargetFn),
    // The check (e, c, tp, r) of a player effect that restricts what that player does to a card: EFFECT_CANNOT_REMOVE.
    PlayerCard(fn(&Duel, EffectId, CardId, u8, u32) -> bool),
    // The check (e, c, sg, tg, ntg, tsg, ntsg) -> (ok, trg, ntrg) of a hand Synchro material, read only by proc_synchro.
    SynchroCheck(SynchroCheckFn),
    // The target (e, re, tp) of EFFECT_ACTIVATE_COST: whether the effect `re` being activated pays its cost.
    ActivateCost(fn(&Duel, EffectId, EffectId, u8) -> bool),
}

pub type SpSummonTargetFn = fn(&Duel, EffectId, CardId, u8, u32, u8, u8, Option<EffectId>) -> bool;
pub type ChangeDamageFn = fn(&mut Duel, EffectId, Option<EffectId>, u32, u32, u8, Option<CardId>) -> u32;

pub type SynchroCheckFn = fn(&Duel, EffectId, Option<CardId>, CardSet, CardSet, CardSet, CardSet, CardSet) -> (bool, CardSet, CardSet);

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Event(OperationFn),
    // A library function that never reaches a prompt, so it runs without the snapshot a rerun would need.
    Promptless(OperationFn),
    Returning(ReturningOperationFn),
    Group(GroupOperationFn),
    // op(e, tsg, ntsg, sg, lv, sc, tp) -> (ok, lvchk): EFFECT_SYNCHRO_MATERIAL_CUSTOM, read only by proc_synchro.
    SynchroMaterial(SynchroMaterialFn),
    // efun(c, e, tp, sg, mg, lc, og, chk): EFFECT_EXTRA_MATERIAL, read only by aux.GetExtraMaterials.
    ExtraMaterial(ExtraMaterialFn),
    // operation(scard, sumtype, tp): when an EFFECT_ADD_CODE, EFFECT_REMOVE_CODE or EFFECT_CHANGE_CODE applies to a summon.
    CodeCondition(CodeConditionFn),
}

pub type CodeConditionFn = fn(&Duel, Option<CardId>, u64, u8) -> bool;

pub type ExtraMaterialFn = fn(&Duel, CardId, EffectId, u8, CardSet, CardSet, CardId, CardSet, i32) -> bool;

pub type SynchroMaterialFn = fn(&Duel, EffectId, CardSet, CardSet, CardSet, u32, CardId, u8) -> (bool, bool);

// effect::value: an integer, or a Lua function whose parameters depend on the effect code reading it.
#[derive(Debug, Clone, Copy)]
pub enum Value {
    Int(i32),
    // The values a Lua function returns regardless of its parameters (a summon procedure's type and zone).
    Values(&'static [i32]),
    // value(e, c) -> (sumtype, zone) of an EFFECT_SPSUMMON_PROC whose zone depends on the card.
    SpSummonProcedure(fn(&Duel, EffectId, CardId) -> (i32, i32)),
    // value(e, re, tp): EFFECT_CANNOT_ACTIVATE.
    EffectPlayer(fn(&Duel, EffectId, EffectId, u8) -> bool),
    // value(e, re, rp, c) that reads the card: EFFECT_INDESTRUCTABLE_EFFECT.
    EffectPlayerCard(fn(&Duel, EffectId, EffectId, u8, CardId) -> bool),
    // value(e, c): get_value(card).
    Card(fn(&Duel, EffectId, CardId) -> i32),
    // value(e), read without a card: EFFECT_DRAW_COUNT.
    Own(fn(&Duel, EffectId) -> i32),
    // value(e, te): EFFECT_IMMUNE_EFFECT.
    Effect(fn(&Duel, EffectId, EffectId) -> bool),
    // value(e, ct): EFFECT_CANNOT_INACTIVATE.
    Chain(fn(&Duel, EffectId, u8) -> bool),
    // value(e, c), called for what it does when a monster's materials are set: EFFECT_MATERIAL_CHECK.
    MaterialCheck(fn(&mut Duel, EffectId, CardId)),
    // value(e, sc, sumtype, tp): EFFECT_CANNOT_BE_MATERIAL.
    Material(fn(&Duel, EffectId, Option<CardId>, u64, u8) -> bool),
    // value(e, se, sp, st, pos, tp): EFFECT_SPSUMMON_CONDITION.
    SpSummonCondition(fn(&Duel, EffectId, Option<EffectId>, u8, u32, u8, u8) -> bool),
    // value(e, c, sc): EFFECT_SYNCHRO_MAT_FROM_HAND.
    MaterialFor(fn(&Duel, EffectId, CardId, CardId) -> bool),
    // value(e, c, rc): EFFECT_XYZ_LEVEL.
    XyzLevel(fn(&Duel, EffectId, CardId, CardId) -> i32),
    // value(e, c, rc) returning several levels: EFFECT_XYZ_LEVEL; one level is read like XyzLevel's, several are each compared whole.
    XyzLevels(fn(&Duel, EffectId, CardId, CardId) -> Vec<i32>),
    // value(e, playerid, uplayer, reason): the zones of EFFECT_MUST_USE_MZONE.
    Zones(fn(&Duel, EffectId, u8, u8, u32) -> i32),
    // value(e, re, r, rp): EFFECT_DESTROY_SUBSTITUTE.
    Reason(fn(&Duel, EffectId, Option<EffectId>, u32, u8) -> bool),
    // value(e, re, r, rp), which may reset the effect: EFFECT_INDESTRUCTABLE_COUNT.
    ReasonMut(fn(&mut Duel, EffectId, Option<EffectId>, u32, u8) -> bool),
    // value(e, re), called for what it does when the card is activated: EFFECT_TRAP_ACT_IN_SET_TURN.
    Activation(fn(&mut Duel, EffectId, EffectId)),
    // value(e, damp, rc): EFFECT_CHANGE_BATTLE_DAMAGE.
    BattleDamage(fn(&Duel, EffectId, u8, CardId) -> i64),
    // value(e, re, val, r, rp, rc), which may reset the effect or add hints: EFFECT_CHANGE_DAMAGE.
    ChangeDamage(ChangeDamageFn),
    // value(e, c, sc), which may register effects on c: EFFECT_HAND_SYNCHRO.
    HandSynchro(fn(&mut Duel, EffectId, CardId, CardId) -> bool),
    // value(chk, summon_type, e, ...) of EFFECT_EXTRA_MATERIAL, which registers and resets effects.
    ExtraMaterial(fn(&mut Duel, EffectId, ExtraMaterialCall) -> CardSet),
    // value(base, extracon, e, tp, eg, ep, ev, re, r, rp, chk): EFFECT_COST_REPLACE, read only by Cost.Replaceable.
    CostReplace(fn(&Duel, EffectId, CostReplaceCall) -> bool),
}

// What Cost.Replaceable passes an EFFECT_COST_REPLACE value besides the replacing effect: its extracon, and the
// activated effect's parameters and chk.
#[derive(Debug, Clone, Copy)]
pub struct CostReplaceCall<'a> {
    pub extracon: CostReplaceCondition,
    pub args: &'a Args,
    pub chk: i32,
}

// extracon(base, e, tp, eg, ep, ev, re, r, rp) of Cost.Replaceable, with the replacing effect first.
pub type CostReplaceCondition = fn(&Duel, EffectId, &Args) -> bool;

// The calls aux.GetExtraMaterials and Link.Operation make: chk 0 (tp, sc) for the extra materials, chk 1 (sg, sc, tp)
// once they are used, chk 2 (eg) when the procedure is done with them.
#[derive(Debug, Clone, Copy)]
pub enum ExtraMaterialCall {
    Get { summon_type: u32, tp: u8, sc: CardId },
    Use { summon_type: u32, sg: CardSet, sc: CardId, tp: u8 },
    Delete { eg: CardSet },
}

impl Default for Value {
    fn default() -> Self {
        Value::Int(0)
    }
}

// effect in effect.h; `id` and `status` are kept in [Effects].
#[derive(Debug, Clone, Copy)]
pub struct Effect {
    pub count_limit: u8,
    pub count_limit_max: u8,
    pub count_flag: u8,
    pub count_hopt_index: u8,
    pub effect_owner: u8,
    pub effect_type: u16,
    pub copy_id: u16,
    pub range: u16,
    pub s_range: u16,
    pub o_range: u16,
    pub reset_count: u16,
    pub active_location: u16,
    pub active_sequence: u16,
    pub code: u32,
    pub flag: [u32; 2],
    pub initial_id: u32,
    pub reset_flag: u32,
    pub count_code: u32,
    pub hint_timing: [u32; 2],
    pub card_type: u32,
    pub active_type: u32,
    pub condition: Option<Condition>,
    pub cost: Option<CostFn>,
    // The cost (e, c, playerid, sumtype) of EFFECT_SPSUMMON_COST and the like, which the core only asks whether it holds.
    pub cost_condition: Option<CostConditionFn>,
    pub target: Target,
    pub value: Value,
    pub operation: Option<Operation>,
    pub category: u64,
    pub owner: CardId,
    pub handler: Option<CardId>,
    pub active_handler: Option<CardId>,
    pub description: u64,
    // The values Effect.SetLabel stores (scripts use at most two) and their count, a field of its own so that it packs
    // with the other small fields instead of padding the values to 24 bytes.
    pub label: [i64; 2],
    pub label_len: u8,
    pub label_object: Option<EffectId>,
    // Where the effect is registered: its keys in a card's container and in the field's, which are never 0.
    pub card_key: Option<NonZeroU64>,
    pub field_key: Option<NonZeroU64>,
}

impl Default for Effect {
    fn default() -> Self {
        Self::new(CardId::default())
    }
}

impl Effect {
    pub fn new(owner: CardId) -> Self {
        Self {
            count_limit: 0,
            count_limit_max: 0,
            count_flag: 0,
            count_hopt_index: 0,
            effect_owner: PLAYER_NONE,
            effect_type: 0,
            copy_id: 0,
            range: 0,
            s_range: 0,
            o_range: 0,
            reset_count: 0,
            active_location: 0,
            active_sequence: 0,
            code: 0,
            flag: [0; 2],
            initial_id: 0,
            reset_flag: 0,
            count_code: 0,
            hint_timing: [0; 2],
            card_type: 0,
            active_type: 0,
            condition: None,
            cost: None,
            cost_condition: None,
            target: Target::None,
            value: Value::Int(0),
            operation: None,
            category: 0,
            owner,
            handler: None,
            active_handler: None,
            description: 0,
            label: [0; 2],
            label_len: 0,
            label_object: None,
            card_key: None,
            field_key: None,
        }
    }

    pub fn set_labels(&mut self, values: &[i64]) {
        self.label_len = 0;
        for (slot, &value) in self.label.iter_mut().zip(values) {
            *slot = value;
            self.label_len += 1;
        }
    }

    pub fn labels(&self) -> &[i64] {
        self.label.get(..usize::from(self.label_len)).unwrap_or_default()
    }

    pub fn is_flag(&self, flag: u32) -> bool {
        self.flag[0] & flag != 0
    }

    pub fn is_flag2(&self, flag: u32) -> bool {
        self.flag[1] & flag != 0
    }

    pub fn is_disable_related(&self) -> bool {
        matches!(
            self.code,
            EFFECT_IMMUNE_EFFECT | EFFECT_DISABLE | EFFECT_CANNOT_DISABLE | EFFECT_FORBIDDEN
        )
    }

    pub fn is_can_be_forbidden(&self) -> bool {
        let ctr = self.code & 0xf0000;
        if self.is_flag(EFFECT_FLAG_CANNOT_DISABLE) && !self.is_flag(EFFECT_FLAG_CANNOT_NEGATE) {
            return false;
        }
        !(self.code == EFFECT_CHANGE_CODE || ctr == EFFECT_COUNTER_PERMIT || ctr == EFFECT_COUNTER_LIMIT)
    }

    pub fn get_client_mode(&self) -> u8 {
        if self.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            EFFECT_CLIENT_MODE_RESOLVE
        } else if self.effect_type & EFFECT_TYPE_ACTIONS == 0 {
            EFFECT_CLIENT_MODE_RESET
        } else {
            EFFECT_CLIENT_MODE_NORMAL
        }
    }

    // The integer value; callers of function values ask for their signature instead.
    pub fn int_value(&self) -> i32 {
        match self.value {
            Value::Int(value) => value,
            Value::Values(values) => values.first().copied().unwrap_or(0),
            _ => 0,
        }
    }
}

// effect::id and effect::status change while effects are only read (is_available), so they are Cells beside the
// effects, which keeps Effect itself Copy and lets clones share the effects' chunks.
#[derive(Debug, Clone, Default)]
pub struct Effects {
    list: Shared<Effect, CHUNK>,
    cells: Vec<Cell<CellChunk>>,
    free: List<EffectId, 32>,
    scratch: Effect,
}

const CHUNK: usize = 32;

pub type CellChunk = [(u32, u16); CHUNK];

impl Effects {
    pub fn push(&mut self, effect: Effect) -> EffectId {
        if let Some(id) = self.free.pop() {
            if let Some(slot) = self.list.get_mut(id.index()) {
                *slot = effect;
            }
            self.set_cell(id, |_| (0, 0));
            return id;
        }
        let id = EffectId::new(self.list.len() as u16);
        if self.list.len().is_multiple_of(CHUNK) {
            self.cells.push(Cell::new([(0, 0); CHUNK]));
        }
        self.list.push(effect);
        id
    }

    pub fn sync_from(&mut self, source: &Self) {
        self.list.sync_from(&source.list);
        self.cells.clone_from(&source.cells);
        self.free.clone_from(&source.free);
    }

    fn cell(&self, effect: EffectId) -> Option<&Cell<(u32, u16)>> {
        let index = effect.index();
        let chunk = self.cells.get(index / CHUNK)?.as_array_of_cells();
        chunk.get(index % CHUNK).filter(|_| index < self.list.len())
    }

    fn all_cells(&self) -> impl Iterator<Item = &Cell<(u32, u16)>> {
        self.cells.iter().flat_map(Cell::as_array_of_cells).take(self.list.len())
    }

    pub fn id(&self, effect: EffectId) -> u32 {
        self.cell(effect).map_or(0, |cell| cell.get().0)
    }

    pub fn status(&self, effect: EffectId) -> u16 {
        self.cell(effect).map_or(0, |cell| cell.get().1)
    }

    pub fn set_id(&self, effect: EffectId, id: u32) {
        self.set_cell(effect, |(_, status)| (id, status));
    }

    pub fn set_status(&self, effect: EffectId, status: u16) {
        self.set_cell(effect, |(id, _)| (id, status));
    }

    fn set_cell(&self, effect: EffectId, update: impl FnOnce((u32, u16)) -> (u32, u16)) {
        if let Some(cell) = self.cell(effect) {
            cell.set(update(cell.get()));
        }
    }

    pub fn save_cells(&self, saved: &mut Vec<CellChunk>) {
        saved.clear();
        saved.extend(self.cells.iter().map(Cell::get));
    }

    pub fn restore_cells(&self, saved: &[CellChunk]) {
        self.cells.iter().zip(saved).for_each(|(cell, &value)| cell.set(value));
    }

    // The ids and statuses of a copy of these effects, for the effects both hold.
    pub fn copy_cells_from(&self, source: &Self) {
        self.all_cells()
            .zip(source.all_cells())
            .for_each(|(cell, value)| cell.set(value.get()));
    }

    // The card effects registered here that are not in `other`, a copy of these effects.
    pub fn added_since<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = EffectId> + 'a {
        let registered = |effects: &Self, index: usize| effects.list.get(index).is_some_and(|effect| effect.card_key.is_some());
        (self.list.unshared(&other.list))
            .filter(move |&index| registered(self, index) && !registered(other, index))
            .map(|index| EffectId::new(index as u16))
    }

    // duel::delete_effect: the slot is reused by the next new effect. Label objects are the only references to an
    // effect that scripts keep, so they are the ones that turn into the deleted object.
    pub fn delete(&mut self, ids: &[EffectId]) {
        let holders: EffectSet = (self.list.iter().enumerate())
            .filter(|(_, effect)| effect.label_object.is_some_and(|label| ids.contains(&label)))
            .map(|(index, _)| EffectId::new(index as u16))
            .collect();
        for holder in holders {
            self[holder].label_object = Some(EffectId::DELETED);
        }
        self.free.extend(ids.iter().copied());
    }

    pub fn is_deleted(&self, id: EffectId) -> bool {
        id == EffectId::DELETED || self.free.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_a_label_object_deleted_after_its_slot_is_reused() {
        let mut effects = Effects::default();
        let holder = effects.push(Effect::new(CardId::default()));
        let deleted = effects.push(Effect::new(CardId::default()));
        effects[holder].label_object = Some(deleted);
        effects.delete(&[deleted]);
        let reused = effects.push(Effect::new(CardId::default()));
        let label = effects[holder].label_object;
        assert_eq!(
            (reused, label.map(|e| effects.is_deleted(e)), effects.is_deleted(reused)),
            (deleted, Some(true), false)
        );
    }
}

// Ids only come from `push`, so the scratch effect is never reached.
impl Index<EffectId> for Effects {
    type Output = Effect;

    fn index(&self, id: EffectId) -> &Effect {
        self.list.get(id.index()).unwrap_or(&self.scratch)
    }
}

impl IndexMut<EffectId> for Effects {
    fn index_mut(&mut self, id: EffectId) -> &mut Effect {
        let Self { list, scratch, .. } = self;
        list.get_mut(id.index()).unwrap_or(scratch)
    }
}

// The containers of effect_container (a multimap from code to effect) on cards and on the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Container {
    Single,
    Field,
    Equip,
    Target,
    XMaterial,
    Aura,
    Ignition,
    Activate,
    TriggerO,
    TriggerF,
    QuickO,
    QuickF,
    Continuous,
}

pub fn container_key(container: Container, card: Option<CardId>, code: u32) -> u64 {
    let card = card.map_or(0, |card| card.index() as u64 + 1);
    ((container as u64) << 48) | (card << 32) | u64::from(code)
}

// Every effect_container of the duel in one sorted list: entries of one key keep insertion order, like a multimap's
// equal range, so lookups are a binary search and iteration needs no allocation. Most lookups find nothing, so bitmaps
// of the codes, the containers of each card and the keys that have entries answer those without searching.
#[derive(Debug, Default)]
pub struct Registry {
    entries: Vec<(u64, EffectId)>,
    codes: [u64; 32],
    cards: [u64; 32],
    keys: [u64; 32],
    changed: Cell<bool>,
}

clone_by_field!(Registry {
    entries,
    codes,
    cards,
    keys,
    changed
});

#[inline]
fn code_bit(code: u32) -> usize {
    (code.wrapping_mul(0x9e37_79b9) >> 21) as usize
}

#[inline]
fn key_bit(key: u64) -> usize {
    (key.wrapping_mul(0x9e37_79b9_7f4a_7c15) >> 53) as usize
}

#[inline]
fn card_bit(key: u64) -> usize {
    (((key >> 32) as u32).wrapping_mul(0x9e37_79b9) >> 21) as usize
}

#[inline]
fn test(bits: &[u64], bit: usize) -> bool {
    bits.get(bit / 64).is_some_and(|word| word & (1 << (bit % 64)) != 0)
}

fn set(bits: &mut [u64], bit: usize, value: bool) {
    if let Some(word) = bits.get_mut(bit / 64) {
        *word = (*word & !(1 << (bit % 64))) | (u64::from(value) << (bit % 64));
    }
}

impl Registry {
    pub fn insert(&mut self, key: u64, effect: EffectId) {
        let at = self.entries.partition_point(|&(other, _)| other <= key);
        self.entries.insert(at, (key, effect));
        self.changed.set(true);
        set(&mut self.codes, code_bit(key as u32), true);
        set(&mut self.cards, card_bit(key), true);
        set(&mut self.keys, key_bit(key), true);
    }

    pub fn remove(&mut self, key: u64, effect: EffectId) {
        let range = self.range(key, key + 1);
        let start = range.start;
        if let Some(offset) = self
            .entries
            .get(range)
            .and_then(|entries| entries.iter().position(|&(_, e)| e == effect))
        {
            self.entries.remove(start + offset);
            self.changed.set(true);
        }
        if self.entries.get(start).is_some_and(|&(other, _)| other == key) {
            return;
        }
        let (code, card) = (code_bit(key as u32), card_bit(key));
        set(
            &mut self.codes,
            code,
            self.entries.iter().any(|&(other, _)| code_bit(other as u32) == code),
        );
        set(
            &mut self.cards,
            card,
            self.entries.iter().any(|&(other, _)| card_bit(other) == card),
        );
        let bit = key_bit(key);
        set(&mut self.keys, bit, self.entries.iter().any(|&(other, _)| key_bit(other) == bit));
    }

    // Makes this equal to `source`, which must have been equal apart from what either changed since they last synced.
    pub fn sync_from(&mut self, source: &Self) {
        if self.changed.take() | source.changed.take() {
            self.clone_from(source);
        }
    }

    // False when no container holds an effect with this code. The lookups that run most check it in an inlined
    // wrapper first, so that the usual lookup of a code no effect has costs one bit test at the call site.
    #[inline]
    pub fn has_code(&self, code: u32) -> bool {
        test(&self.codes, code_bit(code))
    }

    // Equal ranges are short, so the end is found by scanning from the start.
    fn range(&self, from: u64, to: u64) -> std::ops::Range<usize> {
        let start = self.entries.partition_point(|&(key, _)| key < from);
        let len = self
            .entries
            .get(start..)
            .map_or(0, |rest| rest.iter().take_while(|&&(key, _)| key < to).count());
        start..start + len
    }

    fn slice(&self, from: u64, to: u64) -> &[(u64, EffectId)] {
        self.entries.get(self.range(from, to)).unwrap_or_default()
    }

    pub fn equal_range(&self, container: Container, card: Option<CardId>, code: u32) -> impl Iterator<Item = EffectId> + Clone + '_ {
        let key = container_key(container, card, code);
        let entries = if test(&self.keys, key_bit(key)) {
            self.slice(key, key + 1)
        } else {
            &[]
        };
        entries.iter().map(|&(_, effect)| effect)
    }

    // A whole container in multimap order: the code, then insertion.
    pub fn all(&self, container: Container, card: Option<CardId>) -> impl Iterator<Item = (u32, EffectId)> + Clone + '_ {
        let from = container_key(container, card, 0);
        let entries = if test(&self.cards, card_bit(from)) {
            self.slice(from, from + (1 << 32))
        } else {
            &[]
        };
        entries.iter().map(|&(key, effect)| (key as u32, effect))
    }
}

// field_effect in field.h apart from the containers the registry holds. The core keeps the collections in hash sets,
// iterated in address order; these keep insertion order, which only orders the messages of removed hint effects.
#[derive(Debug, Default)]
pub struct FieldEffect {
    pub registry: Registry,
    pub oath: List<(EffectId, Option<EffectId>), 2>,
    pub pheff: List<EffectId, 8>,
    pub cheff: List<EffectId, 2>,
    pub rechargeable: List<EffectId, 8>,
    pub spsummon_count_eff: List<EffectId, 2>,
    pub disable_check_set: CardSet,
    // Per EFFECT_TYPE_GRANT effect, in initial id order, the cards it granted a copy of its label object effect to.
    pub grant_effect: Vec<(EffectId, Vec<(CardId, EffectId)>)>,
}

clone_by_field!(FieldEffect {
    registry,
    oath,
    pheff,
    cheff,
    rechargeable,
    spsummon_count_eff,
    disable_check_set,
    grant_effect
});

impl FieldEffect {
    // clone_from, with the registry synced as [Registry::sync_from] does.
    pub fn sync_from(&mut self, source: &Self) {
        let Self {
            registry,
            oath,
            pheff,
            cheff,
            rechargeable,
            spsummon_count_eff,
            disable_check_set,
            grant_effect,
        } = source;
        self.registry.sync_from(registry);
        self.oath.clone_from(oath);
        self.pheff.clone_from(pheff);
        self.cheff.clone_from(cheff);
        self.rechargeable.clone_from(rechargeable);
        self.spsummon_count_eff.clone_from(spsummon_count_eff);
        self.disable_check_set = *disable_check_set;
        self.grant_effect.clone_from(grant_effect);
    }
}

pub fn remove_item<T: Copy + Default + PartialEq, const N: usize>(list: &mut List<T, N>, item: &T) {
    if let Some(at) = list.iter().position(|other| other == item) {
        list.remove(at);
    }
}

// A custom activity counter (Duel.AddCustomActivityCounter): what does not pass the filter counts.
#[derive(Debug, Clone, Copy)]
pub enum CounterFilter {
    // filter(re, tp, cid) of ACTIVITY_CHAIN.
    Chain(fn(&Duel, EffectId, u8, u8) -> bool),
    // filter(c) of the summon and attack activities.
    Card(fn(&Duel, CardId) -> bool),
}

#[derive(Debug, Clone, Copy)]
pub struct CustomCounter {
    pub id: u32,
    pub activity: u8,
    pub filter: CounterFilter,
    pub amount: [u32; 2],
}

impl Default for CustomCounter {
    fn default() -> Self {
        Self {
            id: 0,
            activity: 0,
            filter: CounterFilter::Card(|_, _| true),
            amount: [0; 2],
        }
    }
}

// tevent in field.h.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Event {
    pub trigger_card: Option<CardId>,
    pub event_cards: Option<CardSet>,
    pub reason_effect: Option<EffectId>,
    pub event_code: u32,
    pub event_value: u32,
    pub reason: u32,
    pub event_player: u8,
    pub reason_player: u8,
    pub global_id: u32,
}

// optarget in field.h.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpTarget {
    pub op_cards: Option<CardSet>,
    pub op_count: u8,
    pub op_player: u8,
    pub op_param: i32,
}

// chain::opmap, keyed by category; only looked up, so its order does not matter. Scripts set a few categories at most.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpInfos([Option<(u64, OpTarget)>; 4]);

impl OpInfos {
    pub fn get(&self, category: u64) -> Option<&OpTarget> {
        self.0.iter().flatten().find(|(key, _)| *key == category).map(|(_, target)| target)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn set(&mut self, category: u64, target: OpTarget) {
        let slot = match self.0.iter().position(|entry| entry.is_some_and(|(key, _)| key == category)) {
            Some(index) => self.0.get_mut(index),
            None => self.0.iter_mut().find(|entry| entry.is_none()),
        };
        debug_assert!(slot.is_some(), "more operation infos than a chain holds");
        if let Some(slot) = slot {
            *slot = Some((category, target));
        }
    }
}

// The card_state the chain snapshots in set_triggering_state, reduced to what CHAININFO reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TriggeringState {
    pub code: u32,
    pub code2: u32,
    pub card_type: u32,
    pub level: u32,
    pub rank: u32,
    pub attribute: u32,
    pub race: u64,
    pub attack: i32,
    pub defense: i32,
    pub setcodes: [u16; 5],
}

// chain in field.h.
#[derive(Debug, Clone, Copy, Default)]
pub struct Chain {
    pub triggering_state: TriggeringState,
    pub evt: Event,
    pub chain_count: u8,
    pub triggering_player: u8,
    pub triggering_controler: u8,
    pub triggering_position: u8,
    pub target_player: u8,
    pub disable_player: u8,
    pub triggering_summon_location: u8,
    pub triggering_summon_proc_complete: bool,
    pub was_just_sent: bool,
    pub chain_id: u16,
    pub triggering_location: u16,
    pub triggering_sequence: u32,
    pub triggering_status: u32,
    pub triggering_summon_type: u32,
    pub replace_op: Option<OperationFn>,
    pub target_param: i32,
    pub flag: u32,
    pub event_id: u32,
    pub triggering_effect: EffectId,
    // Bit i: the chain activity counter at index i counted this link (applied_chain_counters).
    pub applied_chain_counters: u8,
    pub target_cards: Option<CardSet>,
    pub disable_reason: Option<EffectId>,
    pub opinfos: OpInfos,
    pub possibleopinfos: OpInfos,
}

impl Chain {
    pub fn new(triggering_effect: EffectId) -> Self {
        Self {
            triggering_effect,
            ..Self::default()
        }
    }
}

impl Duel {
    // chain::chain_operation_sort, for list::sort (stable).
    pub(crate) fn sort_chains(&self, chains: &mut [Chain]) {
        chains.sort_by_key(|chain| self.effects.id(chain.triggering_effect));
    }

    pub(crate) fn new_effect(&mut self, owner: CardId) -> EffectId {
        self.effects.push(Effect::new(owner))
    }

    pub(crate) fn next_field_id(&self) -> u32 {
        let id = self.infos.field_id.get();
        self.infos.field_id.set(id.wrapping_add(1));
        id
    }

    // Whether a single, equip, field or target effect applies now: range, owner relation, the handler's state and the
    // condition. Actions are never available.
    pub(crate) fn is_available(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        let effect_type = peffect.effect_type;
        if effect_type & EFFECT_TYPE_ACTIONS != 0 {
            return false;
        }
        let relation_ok = |handler: CardId, owner: CardId| {
            let (phandler, powner) = (&self.cards[handler], &self.cards[owner]);
            !((peffect.is_flag(EFFECT_FLAG_OWNER_RELATE) && peffect.is_can_be_forbidden() && powner.is_status(STATUS_FORBIDDEN))
                || (owner == handler && peffect.is_can_be_forbidden() && phandler.get_status(STATUS_FORBIDDEN))
                || (peffect.is_flag(EFFECT_FLAG_OWNER_RELATE)
                    && !peffect.is_flag(EFFECT_FLAG_CANNOT_DISABLE)
                    && powner.is_status(STATUS_DISABLED))
                || (owner == handler && !peffect.is_flag(EFFECT_FLAG_CANNOT_DISABLE) && phandler.get_status(STATUS_DISABLED)))
        };
        let (handler, owner) = (self.get_handler(effect), self.get_owner(effect));
        if effect_type & (EFFECT_TYPE_SINGLE | EFFECT_TYPE_XMATERIAL) != 0 && effect_type & EFFECT_TYPE_FIELD == 0 {
            let phandler = &self.cards[handler];
            let single_range = peffect.is_flag(EFFECT_FLAG_SINGLE_RANGE);
            if (phandler.current.controler == PLAYER_NONE
                && peffect.code != EFFECT_ADD_SETCODE
                && peffect.code != EFFECT_ADD_CODE
                && peffect.code != EFFECT_REVIVE_LIMIT)
                || !self.is_in_range_of_symbolic_mzone(effect, handler)
                || (single_range && !self.in_range(effect, handler))
                || (single_range && !phandler.get_status(STATUS_EFFECT_ENABLED) && !peffect.is_flag(EFFECT_FLAG_IMMEDIATELY_APPLY))
                || (single_range && phandler.current.location & LOCATION_ONFIELD != 0 && !phandler.is_position(POS_FACEUP))
                || !relation_ok(handler, owner)
            {
                return false;
            }
        }
        if effect_type & EFFECT_TYPE_EQUIP != 0 {
            let handler = peffect.handler.unwrap_or_default();
            let phandler = &self.cards[handler];
            if phandler.current.controler == PLAYER_NONE
                || !relation_ok(handler, peffect.owner)
                || (!peffect.is_flag(EFFECT_FLAG_SET_AVAILABLE)
                    && (!phandler.get_status(STATUS_EFFECT_ENABLED) || !phandler.is_position(POS_FACEUP)))
            {
                return false;
            }
        }
        if effect_type & (EFFECT_TYPE_FIELD | EFFECT_TYPE_TARGET) != 0 && !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            let phandler = &self.cards[handler];
            if phandler.current.controler == PLAYER_NONE
                || !self.is_in_range_of_symbolic_mzone(effect, handler)
                || !self.in_range(effect, handler)
                || (!phandler.get_status(STATUS_EFFECT_ENABLED) && !peffect.is_flag(EFFECT_FLAG_IMMEDIATELY_APPLY))
                || (phandler.current.location & LOCATION_ONFIELD != 0 && !phandler.is_position(POS_FACEUP))
                || !relation_ok(handler, owner)
                || phandler.is_status(STATUS_BATTLE_DESTROYED)
            {
                return false;
            }
        }
        let Some(condition) = peffect.condition else {
            return true;
        };
        let result = self.check_condition(condition, &Args::effect(effect));
        let status = self.effects.status(effect);
        if result {
            if status & EFFECT_STATUS_AVAILABLE == 0 {
                self.effects.set_id(effect, self.next_field_id());
            }
            self.effects.set_status(effect, status | EFFECT_STATUS_AVAILABLE);
        } else {
            self.effects.set_status(effect, status & !EFFECT_STATUS_AVAILABLE);
        }
        result
    }

    // count_limit: activations left; count_limit_max: activations allowed.
    pub(crate) fn check_count_limit(&self, effect: EffectId, playerid: u8) -> bool {
        let peffect = &self.effects[effect];
        if !peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) {
            return true;
        }
        if peffect.count_limit == 0 {
            return false;
        }
        if peffect.count_code != 0 || peffect.count_flag != 0 {
            let used = if peffect.count_flag & EFFECT_COUNT_CODE_SINGLE != 0 {
                let fieldid = self.cards[self.get_handler(effect)].fieldid;
                self.get_effect_code(fieldid, peffect.count_flag, peffect.count_hopt_index, PLAYER_NONE)
            } else {
                self.get_effect_code(peffect.count_code, peffect.count_flag, peffect.count_hopt_index, playerid)
            };
            if used >= u32::from(peffect.count_limit_max) {
                return false;
            }
        }
        true
    }

    // Whether an EFFECT_TYPE_ACTIONS effect can be activated, checking EFFECT_FLAG_DAMAGE_STEP and
    // EFFECT_FLAG_SET_AVAILABLE for triggers.
    pub(crate) fn is_activateable(
        &mut self,
        effect: EffectId,
        playerid: u8,
        event: &Event,
        neglect_cond: bool,
        neglect_cost: bool,
        neglect_target: bool,
        neglect_loc: bool,
        neglect_faceup: bool,
    ) -> bool {
        let peffect = &self.effects[effect];
        let effect_type = peffect.effect_type;
        if effect_type & EFFECT_TYPE_ACTIONS == 0 || !self.check_count_limit(effect, playerid) {
            return false;
        }
        let allowed = if peffect.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            self.get_owner_player(effect) == playerid || peffect.is_flag(EFFECT_FLAG_BOTH_SIDE)
        } else if effect_type & EFFECT_TYPE_ACTIVATE != 0 {
            self.is_activateable_activate(effect, playerid, neglect_loc)
        } else if effect_type & EFFECT_TYPE_CONTINUOUS == 0 {
            self.is_activateable_trigger(effect, playerid, neglect_faceup)
        } else {
            self.is_activateable_continuous(effect)
        };
        if !allowed {
            return false;
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.core.reason_effect.set(Some(effect));
        self.core.reason_player.set(playerid);
        let result = (effect_type & EFFECT_TYPE_CONTINUOUS != 0 || self.is_action_check(effect, playerid))
            && self.is_activate_ready(effect, effect, playerid, event, neglect_cond, neglect_cost, neglect_target);
        self.core.reason_effect.set(reason_effect);
        self.core.reason_player.set(reason_player);
        result
    }

    // The damage step and damage calculation forbid effects without EFFECT_FLAG_DAMAGE_STEP/CAL, except at the events
    // of those steps.
    fn damage_step_forbids(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        let code = peffect.code;
        (!(1132..=1149).contains(&code) && self.infos.phase == PHASE_DAMAGE && !peffect.is_flag(EFFECT_FLAG_DAMAGE_STEP))
            || (!(1134..=1136).contains(&code) && self.infos.phase == PHASE_DAMAGE_CAL && !peffect.is_flag(EFFECT_FLAG_DAMAGE_CAL))
    }

    // is_activateable for EFFECT_TYPE_ACTIVATE: zones, and the cards activated from the hand or the turn they were set.
    fn is_activateable_activate(&mut self, effect: EffectId, playerid: u8, neglect_loc: bool) -> bool {
        let peffect = &self.effects[effect];
        let handler = peffect.handler.unwrap_or_default();
        let card = self.cards[handler];
        let data_type = card.data.card_type;
        if card.current.controler != playerid && !peffect.is_flag(EFFECT_FLAG_BOTH_SIDE) {
            return false;
        }
        if self.check_unique_onfield(handler, playerid, LOCATION_SZONE).is_some() {
            return false;
        }
        if data_type & TYPE_COUNTER == 0 && self.damage_step_forbids(effect) && !self.get_cteffect(effect, playerid, false) {
            return false;
        }
        let peffect = &self.effects[effect];
        let limit_zone = peffect.is_flag(EFFECT_FLAG_LIMIT_ZONE);
        // The zone function of EFFECT_FLAG_LIMIT_ZONE (get_value with the event) comes with the cards that set it.
        unported!(
            self,
            (limit_zone && data_type & (TYPE_FIELD | TYPE_PENDULUM) == 0).then_some(effect)
        );
        let value = peffect.int_value();
        if card.current.location == LOCATION_SZONE {
            if card.is_position(POS_FACEUP) || card.equiping_target.is_some() {
                return false;
            }
        } else if !((data_type & TYPE_FIELD != 0 && !limit_zone && value <= 0)
            || (!limit_zone && value & i32::from(LOCATION_FZONE) != 0)
            || (!limit_zone && value & i32::from(LOCATION_HAND) != 0))
        {
            let available = if !limit_zone && value & i32::from(LOCATION_MZONE) != 0 {
                self.get_useable_count(Some(handler), playerid, LOCATION_MZONE, playerid, LOCATION_REASON_TOFIELD, 0xff)
                    .0
                    > 0
            } else if data_type & TYPE_PENDULUM != 0 || (!limit_zone && value & i32::from(LOCATION_PZONE) != 0) {
                self.is_location_useable(playerid, LOCATION_PZONE, 0) || self.is_location_useable(playerid, LOCATION_PZONE, 1)
            } else {
                self.get_useable_count(Some(handler), playerid, LOCATION_SZONE, playerid, LOCATION_REASON_TOFIELD, 0xff)
                    .0
                    > 0
            };
            if !available {
                return false;
            }
        }
        let mut ecode = 0;
        if card.current.location == LOCATION_HAND && !neglect_loc {
            if data_type & TYPE_TRAP != 0 {
                ecode = EFFECT_TRAP_ACT_IN_HAND;
            } else if data_type & TYPE_SPELL != 0 && self.infos.turn_player != playerid {
                if data_type & TYPE_QUICKPLAY == 0 && self.is_affected_by_effect(handler, EFFECT_BECOME_QUICK).is_none() {
                    return false;
                }
                ecode = EFFECT_QP_ACT_IN_NTPHAND;
            }
        } else if card.current.location == LOCATION_SZONE {
            if data_type & TYPE_TRAP != 0 && card.get_status(STATUS_SET_TURN) {
                ecode = EFFECT_TRAP_ACT_IN_SET_TURN;
            }
            if data_type & TYPE_SPELL != 0
                && (data_type & TYPE_QUICKPLAY != 0 || self.is_affected_by_effect(handler, EFFECT_BECOME_QUICK).is_some())
                && card.get_status(STATUS_SET_TURN)
            {
                ecode = EFFECT_QP_ACT_IN_SET_TURN;
            }
        }
        if ecode != 0
            && !self
                .filter_effect(handler, ecode, true)
                .iter()
                .any(|&e| self.check_count_limit(e, playerid))
        {
            return false;
        }
        !self.cards[handler].is_status(STATUS_FORBIDDEN) && self.is_affected_by_effect(handler, EFFECT_CANNOT_TRIGGER).is_none()
    }

    // is_activateable for triggers, ignition and quick effects.
    fn is_activateable_trigger(&self, effect: EffectId, playerid: u8, neglect_faceup: bool) -> bool {
        let peffect = &self.effects[effect];
        let (effect_type, code) = (peffect.effect_type, peffect.code);
        let phandler = self.get_handler(effect);
        let handler_type = self.get_type(phandler);
        if !self.is_flag(DUEL_TRIGGER_WHEN_PRIVATE_KNOWLEDGE)
            && handler_type & TYPE_MONSTER == 0
            && self.get_active_type(effect) & TYPE_MONSTER != 0
        {
            return false;
        }
        if !peffect.is_flag2(EFFECT_FLAG2_CONTINUOUS_EQUIP) && handler_type & TYPE_CONTINUOUS != 0 && handler_type & TYPE_EQUIP != 0 {
            return false;
        }
        let card = &self.cards[phandler];
        let current = card.current;
        if !neglect_faceup && current.location & (LOCATION_ONFIELD | LOCATION_REMOVED) != 0 {
            if !card.is_position(POS_FACEUP) && !peffect.is_flag(EFFECT_FLAG_SET_AVAILABLE) {
                return false;
            }
            if card.is_position(POS_FACEUP) && !card.is_status(STATUS_EFFECT_ENABLED) {
                return false;
            }
        }
        if effect_type & (EFFECT_TYPE_FLIP | EFFECT_TYPE_TRIGGER_F) == 0
            && !(effect_type & EFFECT_TYPE_TRIGGER_O != 0 && effect_type & EFFECT_TYPE_SINGLE != 0)
            && self.damage_step_forbids(effect)
        {
            return false;
        }
        if current.location == LOCATION_OVERLAY
            || (effect_type & EFFECT_TYPE_FIELD != 0
                && current.controler != playerid
                && !peffect.is_flag(EFFECT_FLAG_BOTH_SIDE | EFFECT_FLAG_EVENT_PLAYER))
        {
            return false;
        }
        if !self.is_flag(DUEL_TRIGGER_WHEN_PRIVATE_KNOWLEDGE)
            && !self.is_flag(DUEL_RETURN_TO_DECK_TRIGGERS)
            && (current.location == LOCATION_DECK || (current.location == LOCATION_EXTRA && current.position & POS_FACEDOWN != 0))
            && ((effect_type & EFFECT_TYPE_SINGLE != 0 && code != EVENT_TO_DECK)
                || (effect_type & EFFECT_TYPE_FIELD != 0 && peffect.range & u16::from(LOCATION_DECK | LOCATION_EXTRA) == 0))
        {
            return false;
        }
        !card.is_status(STATUS_FORBIDDEN) && self.is_affected_by_effect(phandler, EFFECT_CANNOT_TRIGGER).is_none()
    }

    fn is_activateable_continuous(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        let effect_type = peffect.effect_type;
        let phandler = self.get_handler(effect);
        let card = &self.cards[phandler];
        let owner = &self.cards[peffect.owner];
        let single_range = effect_type & EFFECT_TYPE_SINGLE != 0 && peffect.is_flag(EFFECT_FLAG_SINGLE_RANGE);
        !((effect_type & EFFECT_TYPE_FIELD != 0 && card.is_status(STATUS_BATTLE_DESTROYED))
            || ((effect_type & EFFECT_TYPE_FIELD != 0 || single_range)
                && card.current.location & LOCATION_ONFIELD != 0
                && (!card.is_position(POS_FACEUP) || !card.is_status(STATUS_EFFECT_ENABLED)))
            || (single_range && !self.in_range(effect, phandler))
            || (peffect.is_flag(EFFECT_FLAG_OWNER_RELATE) && peffect.is_can_be_forbidden() && owner.is_status(STATUS_FORBIDDEN))
            || (phandler == peffect.owner && peffect.is_can_be_forbidden() && card.is_status(STATUS_FORBIDDEN))
            || (peffect.is_flag(EFFECT_FLAG_OWNER_RELATE)
                && !peffect.is_flag(EFFECT_FLAG_CANNOT_DISABLE)
                && owner.is_status(STATUS_DISABLED))
            || (phandler == peffect.owner && !peffect.is_flag(EFFECT_FLAG_CANNOT_DISABLE) && card.is_status(STATUS_DISABLED)))
    }

    // The EFFECT_ACTIVATE_COST effects of `playerid` whose target accepts the activated `effect`.
    pub(crate) fn activate_cost_effects(&self, effect: EffectId, playerid: u8) -> Vec<EffectId> {
        let accepts = |&cost: &EffectId| match self.effects[cost].target {
            Target::ActivateCost(target) => target(self, cost, effect, playerid),
            _ => true,
        };
        self.filter_player_effect(playerid, EFFECT_ACTIVATE_COST, true)
            .into_iter()
            .filter(accepts)
            .collect()
    }

    // The values of EFFECT_CANNOT_ACTIVATE, and the target and cost of EFFECT_ACTIVATE_COST.
    pub(crate) fn is_action_check(&self, effect: EffectId, playerid: u8) -> bool {
        for limit in self.filter_player_effect(playerid, EFFECT_CANNOT_ACTIVATE, true) {
            if self.check_value_activate(limit, effect, playerid) {
                return false;
            }
        }
        // The cost of EFFECT_ACTIVATE_COST is kept as the value, which it has none of.
        self.activate_cost_effects(effect, playerid)
            .into_iter()
            .all(|cost| !matches!(self.effects[cost].value, Value::EffectPlayer(check) if !check(self, cost, effect, playerid)))
    }

    // Condition, cost (chk=0) and target (chk=0) of `effect`, called with `reason_effect` as their `e`.
    pub(crate) fn is_activate_ready(
        &mut self,
        reason_effect: EffectId,
        effect: EffectId,
        playerid: u8,
        event: &Event,
        neglect_cond: bool,
        neglect_cost: bool,
        neglect_target: bool,
    ) -> bool {
        if !self.is_in_range_of_symbolic_mzone(effect, self.get_handler(reason_effect)) {
            return false;
        }
        let peffect = &self.effects[effect];
        let (condition, cost, target, effect_type) = (peffect.condition, peffect.cost, peffect.target, peffect.effect_type);
        let args = Args::event(reason_effect, playerid, event);
        if !neglect_cond && condition.is_some_and(|condition| !self.run_condition(condition, &args)) {
            return false;
        }
        if !neglect_cost
            && effect_type & EFFECT_TYPE_CONTINUOUS == 0
            && cost.is_some_and(|cost| !self.check_script(|duel| cost(duel, &args, 0)))
        {
            return false;
        }
        if !neglect_target
            && let Target::Action(target) = target
            && !self.check_script(|duel| target(duel, &args, 0, None))
        {
            return false;
        }
        true
    }

    pub(crate) fn run_condition(&mut self, condition: Condition, args: &Args) -> bool {
        match condition {
            Condition::Check(condition) => condition(self, args),
            Condition::Mut(condition) => condition(self, args),
        }
    }

    // A condition reached where the duel is only read: one that writes would need its writes carried out of the call.
    pub(crate) fn check_condition(&self, condition: Condition, args: &Args) -> bool {
        match condition {
            Condition::Check(condition) => condition(self, args),
            Condition::Mut(condition) => {
                self.reach_unported(concat!(file!(), ":", line!(), " (a condition that writes, reached by a check)"));
                self.check_mut(|duel| condition(duel, args))
            }
        }
    }

    pub(crate) fn is_condition_check(&mut self, effect: EffectId, playerid: u8, event: &Event) -> bool {
        let phandler = self.get_handler(effect);
        let peffect = &self.effects[effect];
        if peffect.effect_type & EFFECT_TYPE_ACTIVATE == 0
            && self.cards[phandler].current.location & (LOCATION_ONFIELD | LOCATION_REMOVED) != 0
            && !self.cards[phandler].is_position(POS_FACEUP)
        {
            return false;
        }
        if !self.is_in_range_of_symbolic_mzone(effect, phandler) {
            return false;
        }
        let Some(condition) = peffect.condition else { return true };
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        self.core.reason_effect.set(Some(effect));
        self.core.reason_player.set(playerid);
        let result = self.run_condition(condition, &Args::event(effect, playerid, event));
        self.core.reason_effect.set(reason_effect);
        self.core.reason_player.set(reason_player);
        result
    }

    // effect::is_target: whether a field effect applies to the card.
    pub(crate) fn effect_is_target(&self, effect: EffectId, card: CardId) -> bool {
        let peffect = &self.effects[effect];
        let effect_type = peffect.effect_type;
        if effect_type & EFFECT_TYPE_ACTIONS != 0 {
            return false;
        }
        if effect_type & (EFFECT_TYPE_SINGLE | EFFECT_TYPE_EQUIP | EFFECT_TYPE_XMATERIAL) != 0 && effect_type & EFFECT_TYPE_FIELD == 0 {
            return true;
        }
        if effect_type & EFFECT_TYPE_TARGET != 0 && effect_type & EFFECT_TYPE_FIELD == 0 {
            return self.is_fit_target_function(effect, card);
        }
        let target = &self.cards[card];
        if !peffect.is_flag(EFFECT_FLAG_SET_AVAILABLE) && target.current.location & LOCATION_ONFIELD != 0 && !target.is_position(POS_FACEUP)
        {
            return false;
        }
        if !peffect.is_flag(EFFECT_FLAG_IGNORE_RANGE) {
            if target.get_status(STATUS_SUMMONING | STATUS_SUMMON_DISABLED | STATUS_ACTIVATE_DISABLED | STATUS_SPSUMMON_STEP)
                || peffect.is_flag(EFFECT_FLAG_SPSUM_PARAM)
            {
                return false;
            }
            let own = if peffect.is_flag(EFFECT_FLAG_ABSOLUTE_TARGET) {
                target.current.controler == 0
            } else {
                target.current.controler == self.get_handler_player(effect)
            };
            if !target.current.is_location(if own { peffect.s_range } else { peffect.o_range }) {
                return false;
            }
        }
        self.is_fit_target_function(effect, card)
    }

    pub(crate) fn is_fit_target_function(&self, effect: EffectId, card: CardId) -> bool {
        match self.effects[effect].target {
            Target::None => true,
            Target::Card(target) => target(self, effect, card),
            // The core passes only (e, c); the rest are nil.
            Target::SpSummon(target) => target(self, effect, card, PLAYER_NONE, 0, 0, PLAYER_NONE, None),
            Target::Action(_) | Target::PlayerCard(_) | Target::SynchroCheck(_) | Target::ActivateCost(_) => {
                debug_assert!(false, "a field effect has an activation target");
                true
            }
        }
    }

    // effect::get_value(card): the integer, or the first value the function returns.
    pub(crate) fn get_value(&self, effect: EffectId, card: CardId) -> i32 {
        match self.effects[effect].value {
            Value::Int(value) => value,
            Value::Values(values) => values.first().copied().unwrap_or(0),
            Value::SpSummonProcedure(value) => value(self, effect, card).0,
            Value::Card(value) => value(self, effect, card),
            Value::EffectPlayer(_)
            | Value::EffectPlayerCard(_)
            | Value::Effect(_)
            | Value::Chain(_)
            | Value::BattleDamage(_)
            | Value::ChangeDamage(_)
            | Value::Activation(_)
            | Value::MaterialCheck(_)
            | Value::Material(_)
            | Value::SpSummonCondition(_)
            | Value::MaterialFor(_)
            | Value::XyzLevel(_)
            | Value::XyzLevels(_)
            | Value::Zones(_)
            | Value::Reason(_)
            | Value::ReasonMut(_)
            | Value::HandSynchro(_)
            | Value::ExtraMaterial(_)
            | Value::CostReplace(_)
            | Value::Own(_) => {
                debug_assert!(false, "a value read with the wrong parameters");
                0
            }
        }
    }

    // effect::get_value(): the integer, or what the function returns for the effect alone.
    pub(crate) fn get_own_value(&self, effect: EffectId) -> i32 {
        match self.effects[effect].value {
            Value::Int(value) => value,
            Value::Values(values) => values.first().copied().unwrap_or(0),
            Value::Own(value) => value(self, effect),
            _ => {
                debug_assert!(false, "a value read with the wrong parameters");
                0
            }
        }
    }

    // effect::get_value(3) of EFFECT_MUST_USE_MZONE, with or without a card (the card is not passed to the function).
    pub(crate) fn get_zones_value(&self, effect: EffectId, playerid: u8, uplayer: u8, reason: u32) -> u32 {
        match self.effects[effect].value {
            Value::Int(value) => value as u32,
            Value::Zones(value) => value(self, effect, playerid, uplayer, reason) as u32,
            _ => {
                debug_assert!(false, "a zone limit read with the wrong parameters");
                0
            }
        }
    }

    // effect::get_value(2) of EFFECT_CHANGE_BATTLE_DAMAGE with the damaged player and the card dealing the damage.
    pub(crate) fn get_battle_damage_value(&self, effect: EffectId, playerid: u8, reason_card: CardId) -> i64 {
        match self.effects[effect].value {
            Value::Int(value) => i64::from(value),
            Value::BattleDamage(value) => value(self, effect, playerid, reason_card),
            _ => {
                debug_assert!(false, "a battle damage value read with the wrong parameters");
                0
            }
        }
    }

    // effect::get_value(1) of EFFECT_CHANGE_BATTLE_DAMAGE with a player target.
    pub(crate) fn get_player_battle_damage_value(&self, effect: EffectId) -> i64 {
        match self.effects[effect].value {
            Value::Int(value) => i64::from(value),
            _ => {
                debug_assert!(false, "a battle damage value read with the wrong parameters");
                0
            }
        }
    }

    // effect::get_value(5) of EFFECT_CHANGE_DAMAGE with the effect, amount, reason, player and card of the damage.
    pub(crate) fn get_change_damage_value(
        &mut self,
        effect: EffectId,
        reason_effect: Option<EffectId>,
        amount: u32,
        reason: u32,
        reason_player: u8,
        reason_card: Option<CardId>,
    ) -> u32 {
        match self.effects[effect].value {
            Value::Int(value) => value as u32,
            Value::ChangeDamage(value) => value(self, effect, reason_effect, amount, reason, reason_player, reason_card),
            _ => {
                debug_assert!(false, "a damage value read with the wrong parameters");
                0
            }
        }
    }

    // effect::get_value(card, 0, retval): every value the function returns.
    pub(crate) fn get_values(&self, effect: EffectId, card: CardId) -> List<i32, 3> {
        match self.effects[effect].value {
            Value::Values(values) => values.iter().copied().collect(),
            Value::SpSummonProcedure(value) => {
                let (sumtype, zone) = value(self, effect, card);
                List::from_iter([sumtype, zone])
            }
            _ => List::from_iter([self.get_value(effect, card)]),
        }
    }

    pub(crate) fn is_target_player(&self, effect: EffectId, playerid: u8) -> bool {
        let peffect = &self.effects[effect];
        if !peffect.is_flag(EFFECT_FLAG_PLAYER_TARGET) {
            return false;
        }
        if peffect.is_flag(EFFECT_FLAG_ABSOLUTE_TARGET) {
            (peffect.s_range != 0 && playerid == 0) || (peffect.o_range != 0 && playerid == 1)
        } else {
            let own = self.get_handler_player(effect);
            (peffect.s_range != 0 && own == playerid) || (peffect.o_range != 0 && own != playerid)
        }
    }

    pub(crate) fn is_immuned(&self, effect: EffectId, card: CardId) -> bool {
        let immune = |&immune: &EffectId| {
            self.is_available(immune)
                && match self.effects[immune].value {
                    Value::Int(value) => value != 0,
                    Value::Effect(value) => value(self, immune, effect),
                    _ => {
                        debug_assert!(false, "an immunity value read with the wrong parameters");
                        false
                    }
                }
        };
        self.immune_effects
            .iter()
            .filter(|&&(cached, _)| cached == card)
            .any(|(_, immune_effect)| immune(immune_effect))
    }

    pub(crate) fn is_chainable(&self, effect: EffectId, tp: u8) -> bool {
        let peffect = &self.effects[effect];
        if peffect.effect_type & EFFECT_TYPE_ACTIONS == 0 {
            return false;
        }
        let speed = self.get_speed(effect);
        // Curse of Field is the exception.
        if peffect.effect_type & EFFECT_TYPE_ACTIVATE != 0 && speed <= 1 && !peffect.is_flag2(EFFECT_FLAG2_COF) {
            return false;
        }
        if let Some(last) = self.core.current_chain.last() {
            let last_speed = self.get_speed(last.triggering_effect);
            if !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY)
                && peffect.effect_type & EFFECT_TYPE_TRIGGER_O != 0
                && self.cards[self.get_handler(effect)].current.location == LOCATION_HAND
            {
                if last_speed > 2 {
                    return false;
                }
            } else if speed < last_speed {
                return false;
            }
        }
        self.core
            .chain_limit
            .iter()
            .chain(&self.core.chain_limit_p)
            .all(|limit| (limit.function)(self, effect, limit.player, tp, limit.capture))
    }

    // effect::reset: whether a reset of `reset_level` removes the effect. RESET_DISABLE only applies when the owner is
    // the handler.
    pub(crate) fn effect_reset(&mut self, effect: EffectId, reset_level: u32, reset_type: u32) -> bool {
        let tp = self.infos.turn_player;
        let pid = self.get_handler_player(effect);
        let owner_code = self.cards[self.effects[effect].owner].data.code;
        let peffect = &mut self.effects[effect];
        match reset_type {
            RESET_EVENT => {
                if peffect.reset_flag & RESET_EVENT == 0 {
                    return false;
                }
                let mut level = reset_level;
                if Some(peffect.owner) != peffect.handler {
                    level &= !RESET_DISABLE;
                }
                level & 0xffff_0000 & peffect.reset_flag != 0
            }
            RESET_CARD => owner_code == reset_level,
            RESET_PHASE => {
                if peffect.reset_flag & RESET_PHASE == 0 {
                    return false;
                }
                if ((peffect.reset_flag & RESET_SELF_TURN != 0 && pid == tp) || (peffect.reset_flag & RESET_OPPO_TURN != 0 && pid != tp))
                    && reset_level & 0x3ff & peffect.reset_flag != 0
                {
                    peffect.reset_count = peffect.reset_count.wrapping_sub(1);
                }
                peffect.reset_count == 0
            }
            RESET_CODE => {
                peffect.code == reset_level
                    && peffect.effect_type & EFFECT_TYPE_SINGLE != 0
                    && peffect.effect_type & EFFECT_TYPE_ACTIONS == 0
            }
            RESET_COPY => u32::from(peffect.copy_id) == reset_level,
            _ => false,
        }
    }

    pub(crate) fn dec_count(&mut self, effect: EffectId, playerid: u8) {
        let peffect = &self.effects[effect];
        if !peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) || peffect.count_limit == 0 {
            return;
        }
        let (count_code, count_flag, hopt) = (peffect.count_code, peffect.count_flag, peffect.count_hopt_index);
        if (count_code == 0 && count_flag == 0) || peffect.is_flag(EFFECT_FLAG_NO_TURN_RESET) {
            self.effects[effect].count_limit -= 1;
        }
        if count_code != 0 || count_flag != 0 {
            if count_flag & EFFECT_COUNT_CODE_SINGLE != 0 {
                let fieldid = self.cards[self.get_handler(effect)].fieldid;
                self.add_effect_code(fieldid, count_flag, hopt, PLAYER_NONE);
            } else {
                self.add_effect_code(count_code, count_flag, hopt, playerid);
            }
        }
    }

    // Writes only a count that changed, so that a clone keeps sharing the chunks of unused effects.
    pub(crate) fn recharge(&mut self, effect: EffectId) {
        let peffect = &self.effects[effect];
        if peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) && peffect.count_limit != peffect.count_limit_max {
            self.effects[effect].count_limit = peffect.count_limit_max;
        }
    }

    // check_value_condition with the parameters of EFFECT_CANNOT_ACTIVATE: value(e, re, tp).
    pub(crate) fn check_value_activate(&self, effect: EffectId, activated: EffectId, playerid: u8) -> bool {
        match self.effects[effect].value {
            Value::Int(value) => value != 0,
            Value::EffectPlayer(value) => value(self, effect, activated, playerid),
            _ => {
                debug_assert!(false, "an activation limit read with the wrong parameters");
                false
            }
        }
    }

    // check_value_condition with the parameters of EFFECT_CANNOT_INACTIVATE: value(e, ct).
    pub(crate) fn check_value_chain(&self, effect: EffectId, chaincount: u8) -> bool {
        match self.effects[effect].value {
            Value::Int(value) => value != 0,
            Value::Chain(value) => value(self, effect, chaincount),
            _ => {
                debug_assert!(false, "an inactivation limit read with the wrong parameters");
                false
            }
        }
    }

    pub(crate) fn get_speed(&self, effect: EffectId) -> i32 {
        let peffect = &self.effects[effect];
        let effect_type = peffect.effect_type;
        if effect_type & EFFECT_TYPE_ACTIONS == 0 {
            0
        } else if effect_type & (EFFECT_TYPE_TRIGGER_O | EFFECT_TYPE_TRIGGER_F | EFFECT_TYPE_IGNITION) != 0 {
            1
        } else if effect_type & (EFFECT_TYPE_QUICK_O | EFFECT_TYPE_QUICK_F) != 0 {
            2
        } else if effect_type & EFFECT_TYPE_ACTIVATE != 0 {
            let handler = peffect.handler.unwrap_or_default();
            let data_type = self.cards[handler].data.card_type;
            if data_type & TYPE_MONSTER != 0 {
                0
            } else if data_type & TYPE_SPELL != 0 {
                if data_type & TYPE_QUICKPLAY != 0 || self.is_affected_by_effect(handler, EFFECT_BECOME_QUICK).is_some() {
                    2
                } else {
                    1
                }
            } else if data_type & TYPE_COUNTER != 0 {
                3
            } else {
                2
            }
        } else {
            0
        }
    }

    // effect::clone: a copy that is not registered anywhere yet.
    pub(crate) fn clone_effect(&mut self, effect: EffectId) -> EffectId {
        let mut copy = self.effects[effect];
        copy.handler = None;
        copy.card_key = None;
        copy.field_key = None;
        let (id, status) = (self.effects.id(effect), self.effects.status(effect));
        let copy = self.effects.push(copy);
        self.effects.set_id(copy, id);
        self.effects.set_status(copy, status);
        copy
    }

    fn get_real_handler(&self, peffect: &Effect) -> Option<CardId> {
        if peffect.active_handler.is_some() {
            return peffect.active_handler;
        }
        if peffect.effect_type & EFFECT_TYPE_XMATERIAL != 0 {
            let handler = &self.cards[peffect.handler.unwrap_or_default()];
            return handler.overlay_target.or(handler.pre_overlay_target);
        }
        None
    }

    pub(crate) fn get_owner(&self, effect: EffectId) -> CardId {
        let peffect = &self.effects[effect];
        self.get_real_handler(peffect).unwrap_or(peffect.owner)
    }

    pub(crate) fn get_owner_player(&self, effect: EffectId) -> u8 {
        let owner = self.effects[effect].effect_owner;
        if owner != PLAYER_NONE {
            return owner;
        }
        self.cards[self.get_owner(effect)].current.controler
    }

    // An effect not registered yet reads the temporary card, where the core would dereference null.
    pub(crate) fn get_handler(&self, effect: EffectId) -> CardId {
        let peffect = &self.effects[effect];
        self.get_real_handler(peffect).or(peffect.handler).unwrap_or_default()
    }

    pub(crate) fn get_handler_player(&self, effect: EffectId) -> u8 {
        let peffect = &self.effects[effect];
        if peffect.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            return peffect.effect_owner;
        }
        self.cards[self.get_handler(effect)].current.controler
    }

    pub(crate) fn in_range(&self, effect: EffectId, card: CardId) -> bool {
        let peffect = &self.effects[effect];
        if peffect.effect_type & EFFECT_TYPE_XMATERIAL != 0 {
            return self.cards[peffect.handler.unwrap_or_default()].overlay_target.is_some();
        }
        let mut range = peffect.range;
        if range & (LOCATION_MMZONE | LOCATION_EMZONE) != 0 {
            range &= !(LOCATION_MMZONE | LOCATION_EMZONE);
            range |= u16::from(LOCATION_MZONE);
        }
        self.cards[card].current.is_location(range)
    }

    pub(crate) fn is_in_range_of_symbolic_mzone(&self, effect: EffectId, card: CardId) -> bool {
        let range = self.effects[effect].range;
        if range & (LOCATION_MMZONE | LOCATION_EMZONE) == 0 {
            return true;
        }
        let current = &self.cards[card].current;
        !current.is_location(LOCATION_MZONE.into()) || current.is_location(range)
    }

    pub(crate) fn in_range_chain(&self, effect: EffectId, chain: &Chain) -> bool {
        let peffect = &self.effects[effect];
        if peffect.effect_type & EFFECT_TYPE_XMATERIAL != 0 {
            return self.cards[peffect.handler.unwrap_or_default()].overlay_target.is_some();
        }
        peffect.range & chain.triggering_location != 0
    }

    pub(crate) fn set_activate_location(&mut self, effect: EffectId) {
        let current = self.cards[self.get_handler(effect)].current;
        let location = (u16::from(current.location), current.sequence as u16);
        let peffect = &self.effects[effect];
        // Most checks find it unchanged, and a write would make the next script snapshot copy the effect.
        if (peffect.active_location, peffect.active_sequence) != location {
            let peffect = &mut self.effects[effect];
            (peffect.active_location, peffect.active_sequence) = location;
        }
    }

    pub(crate) fn set_active_type(&mut self, effect: EffectId) {
        let mut active_type = self.get_type(self.get_handler(effect));
        if active_type & TYPE_TRAPMONSTER != 0 {
            active_type &= !TYPE_TRAP;
        }
        self.effects[effect].active_type = active_type;
    }

    pub(crate) fn get_active_type(&self, effect: EffectId) -> u32 {
        let peffect = &self.effects[effect];
        if peffect.effect_type & 0x7f0 == 0 {
            return self.get_type(peffect.owner);
        }
        if peffect.active_type != 0 {
            return peffect.active_type;
        }
        let handler = self.get_handler(effect);
        if peffect.effect_type & EFFECT_TYPE_ACTIVATE != 0 && self.cards[handler].data.card_type & TYPE_PENDULUM != 0 {
            TYPE_PENDULUM + TYPE_SPELL
        } else {
            self.get_type(handler)
        }
    }
}
