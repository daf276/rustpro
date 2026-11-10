// Eternal Sunshine
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 28903523;

pub const LISTED_NAMES: &[u32] = &[CARD_ANCIENT_FAIRY_DRAGON];
// Set, with Ancient Fairy Life Dragon (it mentions Ancient Fairy Dragon) on the field and opposing monsters to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (43321985, MZONE),
    (11091375, MZONE | OPPONENT),
    (7359741, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e0.set_condition(duel, aux::stat_change_damage_step_condition);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| afd_count(duel, e.get_handler_player(duel)) as i32 * 500),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER,
    );
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn afdfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_ANCIENT_FAIRY_DRAGON) || (c.lists_code(duel, CARD_ANCIENT_FAIRY_DRAGON) && c.is_monster(duel)))
        && c.is_faceup(duel)
}

fn afd_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(afdfilter, tp, LOCATION_ONFIELD, 0, None)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    duel.get_flag_effect(a.tp, ID) < afd_count(duel, a.tp) && aux::stat_change_damage_step_condition(duel, a)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, (f64::from(tc.get_attack(duel)) / 2.0).ceil() as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        e2.set_value(duel, (f64::from(tc.get_defense(duel)) / 2.0).ceil() as i32);
        tc.register_effect(duel, e2);
        tc.negate_effects(duel, c, RESETS_STANDARD_PHASE_END, false);
    }
    Ok(())
}
