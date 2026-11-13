// Port of libeffect.cpp: Effect.CreateEffect and Effect.GlobalEffect are on Duel, the methods on EffectId.
use crate::card::{CardId, CardSet};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{
    CodeConditionFn, Condition, ConditionFn, CostConditionFn, CostFn, CostReplaceFn, EffectId, ExtraMaterialFn, FusionMaterialFn,
    GroupOperationFn, Operation, OperationFn, ReturningOperationFn, SpSummonTargetFn, SynchroCheckFn, SynchroMaterialFn, Target, TargetFn,
    Value, XyzMaterialFn, ZwEquipFn,
};
use crate::script::Args;

impl Duel {
    pub fn create_effect(&mut self, card: CardId) -> EffectId {
        let effect = self.new_effect(card);
        self.effects[effect].effect_owner = self.core.reason_player.get();
        effect
    }

    pub fn global_effect(&mut self) -> EffectId {
        let effect = self.new_effect(CardId::default());
        self.effects[effect].effect_owner = 0;
        effect
    }
}

impl EffectId {
    // Effect.Clone, renamed so it does not read as Clone::clone of the id.
    pub fn clone_effect(self, duel: &mut Duel) -> EffectId {
        duel.clone_effect(self)
    }

    pub fn reset(self, duel: &mut Duel) {
        let peffect = &duel.effects[self];
        if peffect.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            duel.remove_effect(self);
        } else if let Some(handler) = peffect.handler {
            duel.card_remove_effect(handler, self);
        } else if !duel.core.reseted_effects.contains(&self) {
            duel.core.reseted_effects.push(self);
        }
    }

    pub fn set_description(self, duel: &mut Duel, description: u64) {
        duel.effects[self].description = description;
    }

    pub fn get_description(self, duel: &Duel) -> u64 {
        duel.effects[self].description
    }

    pub fn set_code(self, duel: &mut Duel, code: u32) {
        duel.effects[self].code = code;
    }

    pub fn set_range(self, duel: &mut Duel, range: u32) {
        let mut range = range as u16;
        let (mzone, szone) = (u16::from(LOCATION_MZONE), u16::from(LOCATION_SZONE));
        if range & (LOCATION_MMZONE | LOCATION_EMZONE) == LOCATION_MMZONE | LOCATION_EMZONE {
            range |= mzone;
        }
        if range & mzone != 0 {
            range &= !(LOCATION_MMZONE | LOCATION_EMZONE);
        }
        if range & (LOCATION_FZONE | LOCATION_STZONE | LOCATION_PZONE) == LOCATION_FZONE | LOCATION_STZONE | LOCATION_PZONE {
            range |= szone;
        }
        if range & szone != 0 {
            range &= !(LOCATION_FZONE | LOCATION_STZONE | LOCATION_PZONE);
        }
        duel.effects[self].range = range;
    }

    pub fn set_target_range(self, duel: &mut Duel, s_range: u32, o_range: u32) {
        let peffect = &mut duel.effects[self];
        peffect.s_range = s_range as u16;
        peffect.o_range = o_range as u16;
        peffect.flag[0] &= !EFFECT_FLAG_ABSOLUTE_TARGET;
    }

    pub fn get_target_range(self, duel: &Duel) -> (u32, u32) {
        let peffect = &duel.effects[self];
        (peffect.s_range.into(), peffect.o_range.into())
    }

    pub fn get_range(self, duel: &Duel) -> u32 {
        duel.effects[self].range.into()
    }

    pub fn get_count_code(self, duel: &Duel) -> u32 {
        duel.effects[self].count_code
    }

    // The index of SetCountLimit(1, {code, index}), which keeps a card's effects under one code apart; 0 without a code.
    pub fn get_count_hopt_index(self, duel: &Duel) -> u8 {
        duel.effects[self].count_hopt_index
    }

    pub fn set_absolute_range(self, duel: &mut Duel, player: u8, s_range: u32, o_range: u32) {
        let peffect = &mut duel.effects[self];
        let (s_range, o_range) = (s_range as u16, o_range as u16);
        (peffect.s_range, peffect.o_range) = if player == 0 { (s_range, o_range) } else { (o_range, s_range) };
        peffect.flag[0] |= EFFECT_FLAG_ABSOLUTE_TARGET;
    }

    pub fn set_count_limit(self, duel: &mut Duel, count: u8, code: impl Into<CountCode>) {
        self.set_count_limit_full(duel, count, code, 0);
    }

    // The core rejects a count of 0 and invalid flag combinations with a Lua error; no script passes them.
    pub fn set_count_limit_full(self, duel: &mut Duel, count: u8, code: impl Into<CountCode>, flag: u8) {
        let CountCode(code, hopt_index) = code.into();
        let mut flag = flag;
        if flag & EFFECT_COUNT_CODE_CHAIN != 0 && code == 0 {
            flag |= EFFECT_COUNT_CODE_SINGLE;
        }
        let peffect = &mut duel.effects[self];
        peffect.flag[0] |= EFFECT_FLAG_COUNT_LIMIT;
        peffect.count_limit = count;
        peffect.count_limit_max = count;
        peffect.count_code = code;
        peffect.count_flag = flag;
        peffect.count_hopt_index = if code == 0 { 0 } else { hopt_index };
    }

    pub fn set_reset(self, duel: &mut Duel, reset: u32) {
        self.set_reset_full(duel, reset, 0);
    }

    pub fn set_reset_full(self, duel: &mut Duel, reset: u32, count: u16) {
        let peffect = &mut duel.effects[self];
        peffect.reset_flag = reset_both_turns(reset);
        peffect.reset_count = count.max(1);
    }

    pub fn set_type(self, duel: &mut Duel, effect_type: u32) {
        let mut effect_type = effect_type as u16;
        if effect_type & 0x0ff0 != 0 {
            effect_type |= EFFECT_TYPE_ACTIONS;
        } else {
            effect_type &= !EFFECT_TYPE_ACTIONS;
        }
        if effect_type & (EFFECT_TYPE_ACTIVATE | EFFECT_TYPE_IGNITION | EFFECT_TYPE_QUICK_O | EFFECT_TYPE_QUICK_F) != 0 {
            effect_type |= EFFECT_TYPE_FIELD;
        }
        let peffect = &mut duel.effects[self];
        if effect_type & EFFECT_TYPE_ACTIVATE != 0 {
            peffect.range = u16::from(LOCATION_SZONE) + LOCATION_FZONE + u16::from(LOCATION_HAND);
        }
        if effect_type & EFFECT_TYPE_FLIP != 0 {
            peffect.code = EVENT_FLIP;
            if effect_type & EFFECT_TYPE_TRIGGER_O == 0 {
                effect_type |= EFFECT_TYPE_TRIGGER_F;
            }
        }
        peffect.effect_type = effect_type;
    }

    pub fn set_property(self, duel: &mut Duel, property: u32) {
        self.set_property_full(duel, property, 0);
    }

    pub fn set_property_full(self, duel: &mut Duel, property: u32, property2: u32) {
        let peffect = &mut duel.effects[self];
        peffect.flag[0] = (peffect.flag[0] & 0x4f) | (property & !0x4f);
        peffect.flag[1] = property2;
    }

    pub fn get_property_full(self, duel: &Duel) -> (u32, u32) {
        let flag = &duel.effects[self].flag;
        (flag[0], flag[1])
    }

    pub fn set_label(self, duel: &mut Duel, label: i64) {
        duel.effects[self].set_labels(&[label]);
    }

    // Effect.SetLabel with several values.
    pub fn set_labels(self, duel: &mut Duel, labels: &[i64]) {
        unported!(duel, (labels.len() > duel.effects[self].label.len()).then_some(self));
        duel.effects[self].set_labels(labels);
    }

    pub fn get_labels(self, duel: &Duel) -> &[i64] {
        duel.effects[self].labels()
    }

    pub fn get_category(self, duel: &Duel) -> u64 {
        duel.effects[self].category
    }

    pub fn set_category(self, duel: &mut Duel, category: u64) {
        duel.effects[self].category = category;
    }

    pub fn set_hint_timing(self, duel: &mut Duel, self_timing: u32, opponent_timing: u32) {
        duel.effects[self].hint_timing = [self_timing, opponent_timing];
    }

    pub fn set_condition(self, duel: &mut Duel, condition: ConditionFn) {
        duel.effects[self].condition = Some(Condition::Check(condition));
    }

    // Effect.SetCondition for a condition that writes state other functions read later.
    pub fn set_condition_mut(self, duel: &mut Duel, condition: fn(&mut Duel, &Args) -> bool) {
        duel.effects[self].condition = Some(Condition::Mut(condition));
    }

    pub fn set_target(self, duel: &mut Duel, target: TargetFn) {
        duel.effects[self].target = Target::Action(target);
    }

    // Effect.SetTarget for the card filter (e, c) of a non-activated effect.
    pub fn set_target_card(self, duel: &mut Duel, target: fn(&Duel, EffectId, CardId) -> bool) {
        duel.effects[self].target = Target::Card(target);
    }

    // Effect.SetTarget for EFFECT_ACTIVATE_COST.
    pub fn set_target_activate_cost(self, duel: &mut Duel, target: fn(&Duel, EffectId, EffectId, u8) -> bool) {
        duel.effects[self].target = Target::ActivateCost(target);
    }

    // Effect.SetCost for EFFECT_ACTIVATE_COST: cost(e, re, tp), kept as the value.
    pub fn set_cost_activate(self, duel: &mut Duel, cost: fn(&Duel, EffectId, EffectId, u8) -> bool) {
        duel.effects[self].value = Value::EffectPlayer(cost);
    }

    // Effect.SetTarget for EFFECT_CANNOT_SPECIAL_SUMMON with a target that reads past (e, c).
    pub fn set_target_spsummon(self, duel: &mut Duel, target: SpSummonTargetFn) {
        duel.effects[self].target = Target::SpSummon(target);
    }

    pub fn set_player_card_target(self, duel: &mut Duel, target: fn(&Duel, EffectId, CardId, u8, u32) -> bool) {
        duel.effects[self].target = Target::PlayerCard(target);
    }

    // Effect.SetTarget for the check of a hand Synchro material, which proc_synchro calls with its own parameters.
    pub fn set_synchro_check_target(self, duel: &mut Duel, target: SynchroCheckFn) {
        duel.effects[self].target = Target::SynchroCheck(target);
    }

    pub fn get_target(self, duel: &Duel) -> Target {
        duel.effects[self].target
    }

    pub fn get_condition(self, duel: &Duel) -> Option<Condition> {
        duel.effects[self].condition
    }

    pub fn get_cost(self, duel: &Duel) -> Option<CostFn> {
        duel.effects[self].cost
    }

    pub fn set_cost(self, duel: &mut Duel, cost: CostFn) {
        duel.effects[self].cost = Some(cost);
    }

    // Effect.SetCost for the cost (e, c, playerid, sumtype) of the effects the core checks when a card is summoned or Set.
    pub fn set_cost_condition(self, duel: &mut Duel, cost: CostConditionFn) {
        duel.effects[self].cost_condition = Some(cost);
    }

    pub fn set_value(self, duel: &mut Duel, value: impl Into<Value>) {
        let value = value.into();
        let peffect = &mut duel.effects[self];
        match value {
            Value::Int(_) => peffect.flag[0] &= !EFFECT_FLAG_FUNC_VALUE,
            _ => peffect.flag[0] |= EFFECT_FLAG_FUNC_VALUE,
        }
        peffect.value = value;
    }

    pub fn set_operation(self, duel: &mut Duel, operation: OperationFn) {
        duel.effects[self].operation = Some(Operation::Event(operation));
    }

    // Effect.SetOperation(nil).
    pub fn clear_operation(self, duel: &mut Duel) {
        duel.effects[self].operation = None;
    }

    // Effect.SetOperation for library functions that cannot reach a prompt; a debug build checks they never do.
    pub fn set_promptless_operation(self, duel: &mut Duel, operation: OperationFn) {
        duel.effects[self].operation = Some(Operation::Promptless(operation));
    }

    // Effect.SetOperation for EFFECT_DISABLE_FIELD, whose function returns the zones to disable.
    pub fn set_returning_operation(self, duel: &mut Duel, operation: ReturningOperationFn) {
        duel.effects[self].operation = Some(Operation::Returning(operation));
    }

    // Effect.SetOperation for EFFECT_SPSUMMON_PROC_G, whose function fills the group it is passed.
    pub fn set_group_operation(self, duel: &mut Duel, operation: GroupOperationFn) {
        duel.effects[self].operation = Some(Operation::Group(operation));
    }

    // Effect.SetOperation for EFFECT_SYNCHRO_MATERIAL_CUSTOM, whose function proc_synchro calls with its own parameters.
    pub fn set_synchro_material_operation(self, duel: &mut Duel, operation: SynchroMaterialFn) {
        duel.effects[self].operation = Some(Operation::SynchroMaterial(operation));
    }

    // Effect.SetOperation for EFFECT_EXTRA_MATERIAL, whose function aux.GetExtraMaterials calls with its own parameters.
    pub fn set_extra_material_operation(self, duel: &mut Duel, operation: ExtraMaterialFn) {
        duel.effects[self].operation = Some(Operation::ExtraMaterial(operation));
    }

    // Effect.SetOperation for EFFECT_EXTRA_FUSION_MATERIAL, whose function Fusion.SummonEffOP calls with its own parameters.
    pub fn set_fusion_material_operation(self, duel: &mut Duel, operation: FusionMaterialFn) {
        duel.effects[self].operation = Some(Operation::FusionMaterial(operation));
    }

    // Effect.SetOperation for EFFECT_COST_REPLACE, whose function Cost.Replaceable calls with its own parameters.
    pub fn set_cost_replace_operation(self, duel: &mut Duel, operation: CostReplaceFn) {
        duel.effects[self].operation = Some(Operation::CostReplace(operation));
    }

    // Effect.SetOperation for the effect of aux.AddZWEquipLimit, whose function the Utopia Ray monsters call with their own parameters.
    pub fn set_zw_equip_operation(self, duel: &mut Duel, operation: ZwEquipFn) {
        duel.effects[self].operation = Some(Operation::ZwEquip(operation));
    }

    // Effect.SetOperation for the code effects that apply only to summons, whose function is a condition.
    pub fn set_code_condition(self, duel: &mut Duel, condition: CodeConditionFn) {
        duel.effects[self].operation = Some(Operation::CodeCondition(condition));
    }

    pub fn get_value(self, duel: &Duel) -> Value {
        duel.effects[self].value
    }

    pub fn check_count_limit(self, duel: &Duel, player: u8) -> bool {
        duel.check_count_limit(self, player)
    }

    pub fn use_count_limit(self, duel: &mut Duel, player: u8) {
        duel.dec_count(self, player);
    }

    // Effect.UseCountLimit(player, count, oath_only): with oath_only, only an effect of the once-per-turn-oath kind counts.
    pub fn use_count_limit_full(self, duel: &mut Duel, player: u8, count: u32, oath_only: bool) {
        if !oath_only || duel.effects[self].count_flag & EFFECT_COUNT_CODE_OATH != 0 {
            for _ in 0..count {
                duel.dec_count(self, player);
            }
        }
    }

    pub fn restore_count_limit(self, duel: &mut Duel, player: u8) {
        duel.inc_count(self, player);
    }

    // Effect.SetOperation for an EFFECT_DOUBLE_XYZ_MATERIAL effect, whose function proc_xyz calls with the Xyz Monster and the materials.
    pub fn set_xyz_material_operation(self, duel: &mut Duel, operation: XyzMaterialFn) {
        duel.effects[self].operation = Some(Operation::XyzMaterial(operation));
    }

    pub fn is_activatable(self, duel: &Duel, player: u8, neglect_loc: bool, neglect_target: bool) -> bool {
        duel.check_mut(|duel| {
            let event = duel.nil_event;
            duel.is_activateable(self, player, &event, false, false, neglect_target, neglect_loc, false)
        })
    }

    pub fn get_field_id(self, duel: &Duel) -> u32 {
        duel.effects.id(self)
    }

    pub fn get_label(self, duel: &Duel) -> i64 {
        duel.effects[self].labels().first().copied().unwrap_or(0)
    }

    pub fn set_label_object(self, duel: &mut Duel, object: Option<EffectId>) {
        duel.effects[self].label_object = object;
    }

    pub fn get_label_object(self, duel: &Duel) -> Option<EffectId> {
        duel.effects[self].label_object
    }

    // A group as label object, kept beside the effect.
    pub fn set_label_object_group(self, duel: &mut Duel, group: Option<CardSet>) {
        duel.set_script_group((1 << 62) | self.index() as u64, group);
    }

    pub fn get_label_object_group(self, duel: &Duel) -> Option<CardSet> {
        duel.get_script_group((1 << 62) | self.index() as u64)
    }

    // Deleted: freed at the start of the turn after its reset; a label object stays deleted once its slot is reused.
    pub fn is_deleted(self, duel: &Duel) -> bool {
        duel.effects.is_deleted(self)
    }

    pub fn get_owner(self, duel: &Duel) -> CardId {
        duel.get_owner(self)
    }

    pub fn set_owner_player(self, duel: &mut Duel, player: u8) {
        if player <= 1 {
            duel.effects[self].effect_owner = player;
        }
    }

    pub fn get_activate_location(self, duel: &Duel) -> u32 {
        duel.effects[self].active_location.into()
    }

    pub fn get_owner_player(self, duel: &Duel) -> u8 {
        duel.get_owner_player(self)
    }

    pub fn get_handler_player(self, duel: &Duel) -> u8 {
        duel.get_handler_player(self)
    }

    pub fn get_active_type(self, duel: &Duel) -> u32 {
        duel.get_active_type(self)
    }

    pub fn is_active_type(self, duel: &Duel, card_type: u32) -> bool {
        duel.get_active_type(self) & card_type != 0
    }

    pub fn is_has_property(self, duel: &Duel, property: u32) -> bool {
        property == 0 || duel.effects[self].flag[0] & property != 0
    }

    pub fn is_has_type(self, duel: &Duel, effect_type: u32) -> bool {
        u32::from(duel.effects[self].effect_type) & effect_type != 0
    }

    pub fn get_code(self, duel: &Duel) -> u32 {
        duel.effects[self].code
    }

    pub fn get_handler(self, duel: &Duel) -> CardId {
        duel.get_handler(self)
    }

    pub fn is_has_category(self, duel: &Duel, category: u64) -> bool {
        duel.effects[self].category & category != 0
    }

    pub fn is_activated(self, duel: &Duel) -> bool {
        duel.effects[self].effect_type & 0x7f0 != 0
    }

    pub fn get_operation(self, duel: &Duel) -> Option<Operation> {
        duel.effects[self].operation
    }
}

pub(crate) const fn reset_both_turns(reset: u32) -> u32 {
    if reset & RESET_PHASE != 0 && reset & (RESET_SELF_TURN | RESET_OPPO_TURN) == 0 {
        reset | RESET_SELF_TURN | RESET_OPPO_TURN
    } else {
        reset
    }
}

pub(crate) const fn flag_code(code: u32) -> u32 {
    (code & 0xfff_ffff) | 0x1000_0000
}

// Effect.SetCountLimit's code, or the table {code, index} of a hard once per turn with several effects.
#[derive(Debug, Clone, Copy)]
pub struct CountCode(pub u32, pub u8);

impl From<u32> for CountCode {
    fn from(code: u32) -> Self {
        CountCode(code, 0)
    }
}

impl From<(u32, u8)> for CountCode {
    fn from((code, index): (u32, u8)) -> Self {
        CountCode(code, index)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Int(i32::from(value))
    }
}
