// Cat Shark
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 84224627;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, aux::stat_change_damage_step_condition);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_WATER), 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_rank_below(duel, 4)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let Some(tc) = duel
        .select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(tc.into()), 1, tp, tc.get_base_attack(duel) * 2);
    duel.set_operation_info(0, CATEGORY_DEFCHANGE, Some(tc.into()), 1, tp, tc.get_base_defense(duel) * 2);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, tc.get_base_attack(duel) * 2);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        e2.set_value(duel, tc.get_base_defense(duel) * 2);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
