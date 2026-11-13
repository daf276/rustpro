// Port of effect.h/effect.cpp: effects are plain data in one per-duel arena, their behaviour static function pointers.
use std::cell::Cell;
use std::num::{NonZeroU16, NonZeroU64};
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::card::{CardId, CardSet, MAX_CARDS};
use crate::constants::*;
use crate::duel::Duel;
use crate::list::{self, List, Shared};
use crate::script::{Args, Script};

// The index in the duel's arena, whose slots are reused after a delete as the core reuses memory. Stored plus one.
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

// effect_set: effects collected from containers, sorted by id where the core sorts them. Most lookups find nothing, so
// the list is only made by the first push: an empty set is one word to write, test and drop.
#[derive(Debug, Clone, Default)]
pub struct EffectSet(Option<List<EffectId, 8>>);

impl EffectSet {
    pub fn push(&mut self, effect: EffectId) {
        self.0.get_or_insert_default().push(effect);
    }

    pub fn retain(&mut self, keep: impl FnMut(&EffectId) -> bool) {
        if let Some(effects) = &mut self.0 {
            effects.retain(keep);
        }
    }
}

impl Deref for EffectSet {
    type Target = [EffectId];

    fn deref(&self) -> &[EffectId] {
        self.0.as_deref().unwrap_or_default()
    }
}

impl DerefMut for EffectSet {
    fn deref_mut(&mut self) -> &mut [EffectId] {
        self.0.as_deref_mut().unwrap_or_default()
    }
}

impl Extend<EffectId> for EffectSet {
    fn extend<I: IntoIterator<Item = EffectId>>(&mut self, effects: I) {
        effects.into_iter().for_each(|effect| self.push(effect));
    }
}

impl FromIterator<EffectId> for EffectSet {
    fn from_iter<I: IntoIterator<Item = EffectId>>(effects: I) -> Self {
        let mut set = Self::default();
        set.extend(effects);
        set
    }
}

impl<'a> IntoIterator for &'a EffectSet {
    type Item = &'a EffectId;
    type IntoIter = std::slice::Iter<'a, EffectId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct EffectSetIter(Option<list::IntoIter<EffectId, 8>>);

impl Iterator for EffectSetIter {
    type Item = EffectId;

    fn next(&mut self) -> Option<EffectId> {
        self.0.as_mut()?.next()
    }
}

impl IntoIterator for EffectSet {
    type Item = EffectId;
    type IntoIter = EffectSetIter;

    fn into_iter(self) -> EffectSetIter {
        EffectSetIter(self.0.map(List::into_iter))
    }
}

pub type ConditionFn = fn(&Duel, &Args) -> bool;

// effect::condition; one whose Lua writes persistent state (labels, script state, flag effects) is `Mut`.
#[derive(Debug, Clone, Copy, Hash)]
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

// effect::target: the activation target of actions, or the card filter (e, c) of field effects, by the parameters the code passes.
#[derive(Debug, Clone, Copy, Default, Hash)]
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
pub type ReflectDamageFn = fn(&Duel, EffectId, Option<EffectId>, u32, u32, u8, Option<CardId>) -> bool;
pub type ReflectDamageMutFn = fn(&mut Duel, EffectId, Option<EffectId>, u32, u32, u8, Option<CardId>) -> bool;
pub type ChangeDamageFn = fn(&mut Duel, EffectId, Option<EffectId>, u32, u32, u8, Option<CardId>) -> u32;

pub type SynchroCheckFn = fn(&Duel, EffectId, Option<CardId>, CardSet, CardSet, CardSet, CardSet, CardSet) -> (bool, CardSet, CardSet);

#[derive(Debug, Clone, Copy, Hash)]
pub enum Operation {
    Event(OperationFn),
    // A library function that never reaches a prompt, so it runs without the snapshot a rerun would need.
    Promptless(OperationFn),
    Returning(ReturningOperationFn),
    Group(GroupOperationFn),
    // op(e, tsg, ntsg, sg, lv, sc, tp) -> (ok, lvchk): EFFECT_SYNCHRO_MATERIAL_CUSTOM, read only by proc_synchro.
    SynchroMaterial(SynchroMaterialFn),
    // efun(c, e, tp, sg, mg, lc, og, chk): EFFECT_EXTRA_MATERIAL, read only by aux.GetExtraMaterials; `&mut` for the values it calls.
    ExtraMaterial(ExtraMaterialFn),
    // operation(scard, sumtype, tp): when an EFFECT_ADD_CODE, EFFECT_REMOVE_CODE or EFFECT_CHANGE_CODE applies to a summon.
    CodeCondition(CodeConditionFn),
    // op(e, tc, tp, sg) with the Fusion Summon effect as e: EFFECT_EXTRA_FUSION_MATERIAL, read only by Fusion.SummonEffOP.
    FusionMaterial(FusionMaterialFn),
    // operation(base, extracon, e, tp, ...) of an EFFECT_COST_REPLACE effect, read only by Cost.Replaceable.
    CostReplace(CostReplaceFn),
    // operation(c, e, tp, tc) of the effect aux.AddZWEquipLimit registers, read only by the Utopia Ray monsters that equip a ZW card.
    ZwEquip(ZwEquipFn),
    // op(e, xyz, matg) of an EFFECT_DOUBLE_XYZ_MATERIAL effect, with the Xyz Monster's minimum material count (its `minxyzct`): read only by proc_xyz.
    XyzMaterial(XyzMaterialFn),
}

pub type XyzMaterialFn = fn(&Duel, EffectId, CardId, CardSet, usize) -> bool;

pub type ZwEquipFn = fn(&mut Duel, CardId, EffectId, u8, CardId) -> Script<()>;

pub type CostReplaceFn = fn(&mut Duel, EffectId, CostReplaceCall) -> Script<()>;

pub type CodeConditionFn = fn(&Duel, Option<CardId>, u64, u8) -> bool;

pub type FusionMaterialFn = fn(&mut Duel, EffectId, CardId, u8, &mut CardSet) -> Script<()>;

pub type ExtraMaterialFn = fn(&mut Duel, CardId, EffectId, u8, CardSet, CardSet, CardId, CardSet, i32) -> bool;

pub type SynchroMaterialFn = fn(&Duel, EffectId, CardSet, CardSet, CardSet, u32, CardId, u8) -> (bool, bool);

// effect::value: an integer, or a Lua function whose parameters depend on the effect code reading it.
#[derive(Debug, Clone, Copy, Hash)]
pub enum Value {
    Int(i32),
    // The values a Lua function returns regardless of its parameters (a summon procedure's type and zone).
    Values(&'static [i32]),
    // value(e, c) -> (sumtype, zone) of an EFFECT_SPSUMMON_PROC whose zone depends on the card.
    SpSummonProcedure(fn(&Duel, EffectId, CardId) -> (i32, i32)),
    // value(e, c) -> (minimum Tributes, zones, releasable mask): EFFECT_EXTRA_SUMMON_COUNT.
    ExtraSummon(fn(&Duel, EffectId, CardId) -> (i32, i32, i32)),
    // value(e, re, tp): EFFECT_CANNOT_ACTIVATE.
    EffectPlayer(fn(&Duel, EffectId, EffectId, u8) -> bool),
    // value(e, re, rp, c) that reads the card: EFFECT_INDESTRUCTABLE_EFFECT.
    EffectPlayerCard(fn(&Duel, EffectId, EffectId, u8, CardId) -> bool),
    // value(e, c): get_value(card).
    Card(fn(&Duel, EffectId, CardId) -> i32),
    // value(e, c) that is also read with a nil card, as card::is_affected_by_effect(code, nullptr) does.
    OptionalCard(fn(&Duel, EffectId, Option<CardId>) -> i32),
    // value(e), read without a card: EFFECT_DRAW_COUNT.
    Own(fn(&Duel, EffectId) -> i32),
    // value(e, re, c): EFFECT_CANNOT_SELECT_EFFECT_TARGET.
    EffectCard(fn(&Duel, EffectId, EffectId, CardId) -> bool),
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
    // value(e, c, playerid, uplayer, reason) of an EFFECT_MUST_USE_MZONE without a player target, which gets the card.
    CardZones(fn(&Duel, EffectId, CardId, u8, u8, u32) -> i32),
    // value(e, re, r, rp): EFFECT_DESTROY_SUBSTITUTE.
    Reason(fn(&Duel, EffectId, Option<EffectId>, u32, u8) -> bool),
    // value(e, re, r, rp), which may reset the effect: EFFECT_INDESTRUCTABLE_COUNT.
    ReasonMut(fn(&mut Duel, EffectId, Option<EffectId>, u32, u8) -> bool),
    // value(e, re), called for what it does when the card is activated: EFFECT_TRAP_ACT_IN_SET_TURN.
    Activation(fn(&mut Duel, EffectId, EffectId)),
    // value(e, re, tp): EFFECT_LEFT_SPSUMMON_COUNT.
    SpSummonCount(fn(&Duel, EffectId, Option<EffectId>, u8) -> i32),
    // value(e, damp, rc): EFFECT_CHANGE_BATTLE_DAMAGE.
    BattleDamage(fn(&Duel, EffectId, u8, CardId) -> i64),
    // value(e, re, val, r, rp, rc), which may reset the effect or add hints: EFFECT_CHANGE_DAMAGE.
    ChangeDamage(ChangeDamageFn),
    // value(e, re, val, r, rp, rc): EFFECT_REFLECT_DAMAGE.
    ReflectDamage(ReflectDamageFn),
    // value(e, re, val, r, rp, rc), which may add hints or register flag effects: EFFECT_REFLECT_DAMAGE.
    ReflectDamageMut(ReflectDamageMutFn),
    // value(e, c, sc), which may register effects on c: EFFECT_HAND_SYNCHRO.
    HandSynchro(fn(&mut Duel, EffectId, CardId, CardId) -> bool),
    // value(chk, summon_type, e, ...) of EFFECT_EXTRA_MATERIAL, which registers and resets effects.
    ExtraMaterial(fn(&mut Duel, EffectId, ExtraMaterialCall) -> CardSet),
    // value(base, extracon, e, tp, eg, ep, ev, re, r, rp, chk): EFFECT_COST_REPLACE, read only by Cost.Replaceable.
    CostReplace(fn(&Duel, EffectId, CostReplaceCall) -> bool),
    // value(tc, c, tp) of the effect aux.AddZWEquipLimit registers, read only by the Utopia Ray monsters that equip a ZW card.
    ZwEquip(fn(&Duel, CardId, CardId, u8) -> bool),
}

// What Cost.Replaceable passes an EFFECT_COST_REPLACE value besides the replacing effect.
#[derive(Debug, Clone, Copy)]
pub struct CostReplaceCall<'a> {
    pub extracon: CostReplaceCondition,
    pub args: &'a Args,
    pub chk: i32,
}

// extracon(base, e, tp, eg, ep, ev, re, r, rp) of Cost.Replaceable, with the replacing effect first.
pub type CostReplaceCondition = fn(&Duel, EffectId, &Args) -> bool;

// The calls of aux.GetExtraMaterials and Link.Operation: chk 0 to get the materials, chk 1 once used, chk 2 when done.
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
#[derive(Debug, Clone, Copy, Default)]
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
    // The values Effect.SetLabel stores (up to eight) and their count, kept apart so that it packs with the small fields.
    pub label: [i64; 8],
    pub label_len: u8,
    pub label_object: Option<EffectId>,
    // Where the effect is registered: its keys in a card's container and in the field's, which are never 0.
    pub card_key: Option<NonZeroU64>,
    pub field_key: Option<NonZeroU64>,
}

impl Effect {
    pub fn new(owner: CardId) -> Self {
        Self {
            owner,
            effect_owner: PLAYER_NONE,
            ..Self::default()
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
    #[inline]
    pub fn int_value(&self) -> i32 {
        match self.value {
            Value::Int(value) => value,
            Value::Values(values) => values.first().copied().unwrap_or(0),
            _ => 0,
        }
    }
}

// effect::id and effect::status change while effects are only read, so they are Cells beside the Copy effects' shared chunks.
#[derive(Debug, Default)]
pub struct Effects {
    list: Shared<Effect, CHUNK>,
    cells: Vec<Cell<CellChunk>>,
    // Bit i: a cell of chunk i changed since a copy last synced; the last bit stands for every chunk from it on.
    cells_changed: Cell<u64>,
    free: List<EffectId, 32>,
    scratch: Effect,
}

clone_by_field!(Effects {
    list,
    cells,
    cells_changed,
    free,
    scratch
});

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

    // Makes this equal to `source`, which must differ only in what either marked since they last synced.
    pub fn sync_from(&mut self, source: &Self) {
        self.list.sync_from(&source.list);
        self.cells.truncate(source.cells.len());
        let changed = self.cells_changed.take() | source.cells_changed.take();
        for (chunk, (cell, value)) in self.cells.iter().zip(&source.cells).enumerate() {
            if changed >> chunk.min(63) & 1 != 0 {
                cell.set(value.get());
            }
        }
        self.cells
            .extend_from_slice(source.cells.get(self.cells.len()..).unwrap_or_default());
        self.free.clone_from(&source.free);
    }

    // The effect at `index`, which a script kept as a number in chain data; only right while that effect lives, as a
    // deleted effect's slot is reused.
    pub fn at(&self, index: usize) -> Option<EffectId> {
        let id = EffectId::new(u16::try_from(index).ok()?);
        (index < self.list.len() && !self.free.contains(&id)).then_some(id)
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
            let value = update(cell.get());
            if value != cell.get() {
                cell.set(value);
                self.cells_changed
                    .set(self.cells_changed.get() | 1 << (effect.index() / CHUNK).min(63));
            }
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
        self.cells_changed.set(u64::MAX);
    }

    // The card effects registered here that are not in `other`, a copy of these effects.
    pub fn added_since<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = EffectId> + 'a {
        let registered = |effects: &Self, index: usize| effects.list.get(index).is_some_and(|effect| effect.card_key.is_some());
        (self.list.unshared(&other.list))
            .filter(move |&index| registered(self, index) && !registered(other, index))
            .map(|index| EffectId::new(index as u16))
    }

    // duel::delete_effect; label objects, the only effect references scripts keep, turn into the deleted object.
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

    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.list.iter()
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

    #[test]
    fn should_list_the_registry_like_a_sorted_multimap() {
        let mut cards = crate::card::Cards::default();
        let cards: Vec<CardId> = (0..2).filter_map(|_| cards.push(Default::default())).collect();
        let (first, second) = (cards.first().copied(), cards.last().copied());
        let groups = [
            (Container::Single, first),
            (Container::Single, second),
            (Container::Field, first),
            (Container::XMaterial, second),
            (Container::Aura, None),
            (Container::QuickO, None),
        ];
        let (mut registry, mut model, mut state) = (Registry::default(), Vec::new(), 7u32);
        for step in 0..4000 {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let (container, card) = groups.get((state >> 8) as usize % groups.len()).copied().unwrap();
            let (code, key) = (state >> 27, container_key(container, card, state >> 27));
            match model.iter().position(|&(other, _)| other == key).filter(|_| state & 3 != 0) {
                Some(at) => {
                    let (_, effect) = model.remove(at);
                    registry.remove(key, effect);
                }
                None => {
                    let at = model.partition_point(|&(other, _)| other <= key);
                    model.insert(at, (key, EffectId::new(step)));
                    registry.insert(key, EffectId::new(step));
                }
            }
            let listed = |keys: std::ops::Range<u64>| {
                let entries = model.iter().filter(|&&(other, _)| keys.contains(&other));
                entries.map(|&(_, effect)| effect).collect::<Vec<_>>()
            };
            let from = container_key(container, card, 0);
            assert!(registry.equal_range(container, card, code).eq(listed(key..key + NEXT_KEY)));
            assert!(
                registry
                    .all(container, card)
                    .map(|(_, effect)| effect)
                    .eq(listed(from..from + NEXT_CARD))
            );
            if let Some(card) = card {
                let of_card = |&(other, _): &(u64, EffectId)| (other >> 48) % 512 == (key >> 48) % 512;
                assert!(
                    registry
                        .of_card(card)
                        .eq(model.iter().copied().filter(of_card).map(|(_, effect)| effect))
                );
            }
            assert_eq!(
                registry.has_code(code),
                model.iter().any(|&(other, _)| (other >> 16) as u32 == code)
            );
        }
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

// The key of an equal range, never 0: the container, the card plus one and the code, above 16 bits left for an effect.
pub fn container_key(container: Container, card: Option<CardId>, code: u32) -> u64 {
    let card = card.map_or(0, |card| card.index() as u64 + 1);
    ((container as u64) << 57) | (card << 48) | (u64::from(code) << 16)
}

const NEXT_KEY: u64 = 1 << 16;
const NEXT_CARD: u64 = 1 << 48;

// Every effect_container in one list of multimap equal ranges, grouped into a bucket per card and per container of the
// field; counts of the codes and keys answer most misses.
#[derive(Debug)]
pub struct Registry {
    // A key with its effect in the low bits; sorted by key within a bucket, an equal range in insertion order.
    entries: Vec<u64>,
    // Bucket b holds the entries starts[b]..starts[b + 1].
    starts: [u16; BUCKETS + 1],
    codes: Counts,
    hashes: Counts,
    changed: Cell<bool>,
}

clone_by_field!(Registry {
    entries,
    starts,
    codes,
    hashes,
    changed
});

impl Default for Registry {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            starts: [0; BUCKETS + 1],
            codes: Counts([0; 128]),
            hashes: Counts([0; 128]),
            changed: Cell::new(false),
        }
    }
}

const BUCKETS: usize = MAX_CARDS + Container::Continuous as usize + 1;

fn bucket(key: u64) -> usize {
    match (key >> 48) as usize % 512 {
        0 => MAX_CARDS + (key >> 57) as usize,
        card => card - 1,
    }
}

#[inline]
fn code_slot(code: u32) -> usize {
    (code.wrapping_mul(0x9e37_79b9) >> 21) as usize
}

#[inline]
fn key_slot(key: u64) -> usize {
    (key.wrapping_mul(0x9e37_79b9_7f4a_7c15) >> 53) as usize
}

// How many entries hash to each of 2048 slots, in 4 bits; a count that reaches 15 stays, so its slot never reads empty again.
#[derive(Debug, Clone)]
struct Counts([u64; 128]);

impl Counts {
    #[inline]
    fn any(&self, slot: usize) -> bool {
        self.0.get(slot / 16).is_some_and(|word| (word >> (slot % 16 * 4)) & 15 != 0)
    }

    fn count(&mut self, slot: usize, added: bool) {
        let one = 1 << (slot % 16 * 4);
        if let Some(word) = self.0.get_mut(slot / 16)
            && *word & (15 * one) != 15 * one
        {
            *word = if added { *word + one } else { *word - one };
        }
    }
}

impl Registry {
    pub fn insert(&mut self, key: u64, effect: EffectId) {
        let at = self.range(key, key + NEXT_KEY).end;
        self.entries.insert(at, key | u64::from(effect.0.get()));
        self.adjust(key, true);
    }

    pub fn remove(&mut self, key: u64, effect: EffectId) {
        let entry = key | u64::from(effect.0.get());
        if let Some(at) = self.range(key, key + NEXT_KEY).find(|&at| self.entries.get(at) == Some(&entry)) {
            self.entries.remove(at);
            self.adjust(key, false);
        }
    }

    fn adjust(&mut self, key: u64, added: bool) {
        for start in self.starts.iter_mut().skip(bucket(key) + 1) {
            *start = if added { *start + 1 } else { *start - 1 };
        }
        self.codes.count(code_slot((key >> 16) as u32), added);
        self.hashes.count(key_slot(key), added);
        self.changed.set(true);
    }

    // Makes this equal to `source`, which must have been equal apart from what either changed since they last synced.
    pub fn sync_from(&mut self, source: &Self) {
        if self.changed.take() | source.changed.take() {
            self.clone_from(source);
        }
    }

    // False when no container holds this code; hot lookups test it in an inlined wrapper, so a miss costs one test.
    #[inline]
    pub fn has_code(&self, code: u32) -> bool {
        self.codes.any(code_slot(code))
    }

    // The entries from `from` up to `to`, keys of one bucket. Equal ranges are short, so the end is found by scanning.
    fn range(&self, from: u64, to: u64) -> std::ops::Range<usize> {
        let bucket = bucket(from);
        let Some(&[start, end]) = self.starts.get(bucket..bucket + 2) else {
            return 0..0;
        };
        let entries = self.entries.get(start.into()..end.into()).unwrap_or_default();
        let first = entries.partition_point(|&entry| entry < from);
        let len = entries
            .get(first..)
            .map_or(0, |rest| rest.iter().take_while(|&&entry| entry < to).count());
        let first = usize::from(start) + first;
        first..first + len
    }

    fn listed(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = (u32, EffectId)> + Clone + '_ {
        let entries = self.entries.get(range).unwrap_or_default();
        let effect = |entry: u64| NonZeroU16::new(entry as u16).map_or_else(EffectId::default, EffectId);
        entries.iter().map(move |&entry| ((entry >> 16) as u32, effect(entry)))
    }

    pub fn equal_range(&self, container: Container, card: Option<CardId>, code: u32) -> impl Iterator<Item = EffectId> + Clone + '_ {
        let key = container_key(container, card, code);
        let range = if self.hashes.any(key_slot(key)) {
            self.range(key, key + NEXT_KEY)
        } else {
            0..0
        };
        self.listed(range).map(|(_, effect)| effect)
    }

    // A whole container in multimap order: the code, then insertion.
    pub fn all(&self, container: Container, card: Option<CardId>) -> impl Iterator<Item = (u32, EffectId)> + Clone + '_ {
        let from = container_key(container, card, 0);
        self.listed(self.range(from, from + NEXT_CARD))
    }

    // Every container of a card, in the order of [Container].
    pub fn of_card(&self, card: CardId) -> impl Iterator<Item = EffectId> + Clone + '_ {
        let range = self.range(container_key(Container::Single, Some(card), 0), u64::MAX);
        self.listed(range).map(|(_, effect)| effect)
    }
}

// field_effect apart from the registry; insertion order instead of the core's hash sets only orders removed hint messages.
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

    pub fn track(&mut self, effect: EffectId, flag: u32, reset_flag: u32, reason: Option<EffectId>) {
        if flag & EFFECT_FLAG_OATH != 0 {
            self.oath.push((effect, reason));
        }
        if reset_flag & RESET_PHASE != 0 {
            self.pheff.push(effect);
        }
        if reset_flag & RESET_CHAIN != 0 {
            self.cheff.push(effect);
        }
        if flag & EFFECT_FLAG_COUNT_LIMIT != 0 {
            self.rechargeable.push(effect);
        }
    }

    pub fn untrack(&mut self, effect: EffectId, flag: u32, reset_flag: u32) {
        if flag & EFFECT_FLAG_OATH != 0 {
            self.oath.retain(|&(oath, _)| oath != effect);
        }
        if reset_flag & RESET_PHASE != 0 {
            remove_item(&mut self.pheff, &effect);
        }
        if reset_flag & RESET_CHAIN != 0 {
            remove_item(&mut self.cheff, &effect);
        }
        if flag & EFFECT_FLAG_COUNT_LIMIT != 0 {
            remove_item(&mut self.rechargeable, &effect);
        }
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
pub struct OpInfos(pub(crate) [Option<(u64, OpTarget)>; 4]);

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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
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
        let effect = self.effects.push(Effect::new(owner));
        // A reused slot would otherwise inherit the deleted effect's group, where Lua has nil.
        effect.set_label_object_group(self, None);
        effect
    }

    pub(crate) fn next_field_id(&self) -> u32 {
        let id = self.infos.field_id.get();
        self.infos.field_id.set(id.wrapping_add(1));
        id
    }

    // Whether a single, equip, field or target effect applies now; actions never do.
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
        let reason_effect = self.core.reason_effect.replace(Some(effect));
        let reason_player = self.core.reason_player.replace(playerid);
        let result = (effect_type & EFFECT_TYPE_CONTINUOUS != 0 || self.is_action_check(effect, playerid))
            && self.is_activate_ready(effect, effect, playerid, event, neglect_cond, neglect_cost, neglect_target);
        self.core.reason_effect.set(reason_effect);
        self.core.reason_player.set(reason_player);
        result
    }

    // The damage step and calculation forbid effects without EFFECT_FLAG_DAMAGE_STEP/CAL, except at their own events.
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
        // The cost (e, te, tp) of EFFECT_ACTIVATE_COST is stored as its value, which it otherwise lacks.
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
        let reason_effect = self.core.reason_effect.replace(Some(effect));
        let reason_player = self.core.reason_player.replace(playerid);
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
        let peffect = &self.effects[effect];
        match peffect.value {
            Value::Int(_) | Value::Values(_) => peffect.int_value(),
            Value::SpSummonProcedure(value) => value(self, effect, card).0,
            Value::ExtraSummon(value) => value(self, effect, card).0,
            Value::Card(value) => value(self, effect, card),
            Value::OptionalCard(value) => value(self, effect, Some(card)),
            Value::EffectPlayer(_)
            | Value::EffectPlayerCard(_)
            | Value::EffectCard(_)
            | Value::Effect(_)
            | Value::Chain(_)
            | Value::SpSummonCount(_)
            | Value::BattleDamage(_)
            | Value::ChangeDamage(_)
            | Value::ReflectDamage(_)
            | Value::ReflectDamageMut(_)
            | Value::Activation(_)
            | Value::MaterialCheck(_)
            | Value::Material(_)
            | Value::SpSummonCondition(_)
            | Value::MaterialFor(_)
            | Value::XyzLevel(_)
            | Value::XyzLevels(_)
            | Value::Zones(_)
            | Value::CardZones(_)
            | Value::Reason(_)
            | Value::ReasonMut(_)
            | Value::HandSynchro(_)
            | Value::ExtraMaterial(_)
            | Value::CostReplace(_)
            | Value::ZwEquip(_)
            | Value::Own(_) => misread!(self, "a value"),
        }
    }

    // effect::get_value(nullptr): the integer, or what the function returns for a nil card.
    pub(crate) fn get_value_nil(&self, effect: EffectId, card: CardId) -> i32 {
        match self.effects[effect].value {
            Value::OptionalCard(value) => value(self, effect, None),
            Value::Int(_) | Value::Values(_) => self.get_value(effect, card),
            _ => {
                unported!(self);
                self.get_value(effect, card)
            }
        }
    }

    // effect::get_value(): the integer, or what the function returns for the effect alone.
    pub(crate) fn get_own_value(&self, effect: EffectId) -> i32 {
        let peffect = &self.effects[effect];
        match peffect.value {
            Value::Int(_) | Value::Values(_) => peffect.int_value(),
            Value::Own(value) => value(self, effect),
            _ => misread!(self, "a value"),
        }
    }

    // effect::get_value(3) of EFFECT_MUST_USE_MZONE; the card is passed only for an effect without a player target.
    pub(crate) fn get_zones_value(&self, effect: EffectId, card: Option<CardId>, playerid: u8, uplayer: u8, reason: u32) -> u32 {
        match self.effects[effect].value {
            Value::Int(value) => value as u32,
            Value::Zones(value) => value(self, effect, playerid, uplayer, reason) as u32,
            Value::CardZones(value) => match card {
                Some(card) => value(self, effect, card, playerid, uplayer, reason) as u32,
                None => misread!(self, "a zone limit"),
            },
            _ => misread!(self, "a zone limit"),
        }
    }

    // effect::get_value(2) of EFFECT_CHANGE_BATTLE_DAMAGE with the damaged player and the card dealing the damage.
    pub(crate) fn get_battle_damage_value(&self, effect: EffectId, playerid: u8, reason_card: CardId) -> i64 {
        match self.effects[effect].value {
            Value::Int(value) => i64::from(value),
            Value::BattleDamage(value) => value(self, effect, playerid, reason_card),
            _ => misread!(self, "a battle damage value"),
        }
    }

    // effect::get_value(1) of EFFECT_CHANGE_BATTLE_DAMAGE with a player target.
    pub(crate) fn get_player_battle_damage_value(&self, effect: EffectId) -> i64 {
        match self.effects[effect].value {
            Value::Int(value) => i64::from(value),
            _ => misread!(self, "a battle damage value"),
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
            _ => misread!(self, "a damage value"),
        }
    }

    // check_value_condition(5) of EFFECT_REFLECT_DAMAGE with the effect, amount, reason, player and card of the damage.
    pub(crate) fn check_value_reflect_damage(
        &mut self,
        effect: EffectId,
        reason_effect: Option<EffectId>,
        amount: u32,
        reason: u32,
        reason_player: u8,
        reason_card: Option<CardId>,
    ) -> bool {
        match self.effects[effect].value {
            Value::Int(value) => value != 0,
            Value::ReflectDamage(value) => value(self, effect, reason_effect, amount, reason, reason_player, reason_card),
            Value::ReflectDamageMut(value) => value(self, effect, reason_effect, amount, reason, reason_player, reason_card),
            _ => misread!(self, "a damage reflection"),
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
            Value::ExtraSummon(value) => {
                let (min, zone, releasable) = value(self, effect, card);
                List::from_iter([min, zone, releasable])
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
                    _ => misread!(self, "an immunity value"),
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

    // effect::reset: whether the reset removes the effect; RESET_DISABLE only applies when the owner is the handler.
    pub(crate) fn effect_reset(&mut self, effect: EffectId, reset_level: u32, reset_type: u32) -> bool {
        let tp = self.infos.turn_player;
        let pid = self.get_handler_player(effect);
        let owner_code = self.cards[self.effects[effect].owner].data.code;
        let peffect = &self.effects[effect];
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
                let mut reset_count = peffect.reset_count;
                if ((peffect.reset_flag & RESET_SELF_TURN != 0 && pid == tp) || (peffect.reset_flag & RESET_OPPO_TURN != 0 && pid != tp))
                    && reset_level & 0x3ff & peffect.reset_flag != 0
                {
                    reset_count = reset_count.wrapping_sub(1);
                    self.effects[effect].reset_count = reset_count;
                }
                reset_count == 0
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

    pub(crate) fn inc_count(&mut self, effect: EffectId, playerid: u8) {
        let peffect = &self.effects[effect];
        if !peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) || peffect.count_limit == 0 {
            return;
        }
        let (count_code, count_flag, hopt) = (peffect.count_code, peffect.count_flag, peffect.count_hopt_index);
        if ((count_code == 0 && count_flag == 0) || peffect.is_flag(EFFECT_FLAG_NO_TURN_RESET))
            && peffect.count_limit != peffect.count_limit_max
        {
            self.effects[effect].count_limit += 1;
        }
        if count_code != 0 || count_flag != 0 {
            if count_flag & EFFECT_COUNT_CODE_SINGLE != 0 {
                let fieldid = self.cards[self.get_handler(effect)].fieldid;
                self.dec_effect_code(fieldid, count_flag, hopt, PLAYER_NONE);
            } else {
                self.dec_effect_code(count_code, count_flag, hopt, playerid);
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
            _ => misread!(self, "an activation limit"),
        }
    }

    // effect::get_value(2) with the parameters of EFFECT_LEFT_SPSUMMON_COUNT: value(e, re, tp).
    pub(crate) fn get_value_spsummon_count(&self, effect: EffectId, reason_effect: Option<EffectId>, playerid: u8) -> i32 {
        match self.effects[effect].value {
            Value::Int(value) => value,
            Value::SpSummonCount(value) => value(self, effect, reason_effect, playerid),
            _ => misread!(self, "a spsummon count"),
        }
    }

    // check_value_condition with the parameters of EFFECT_CANNOT_INACTIVATE: value(e, ct).
    pub(crate) fn check_value_chain(&self, effect: EffectId, chaincount: u8) -> bool {
        match self.effects[effect].value {
            Value::Int(value) => value != 0,
            Value::Chain(value) => value(self, effect, chaincount),
            _ => misread!(self, "an inactivation limit"),
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
        copy.set_label_object_group(self, effect.get_label_object_group(self));
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
