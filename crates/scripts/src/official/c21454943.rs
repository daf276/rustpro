// Psychic Commander
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 21454943;

// Copies on the field to fight the opponent's attackers and Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition_mut(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &mut Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    if duel.get_current_phase() != PHASE_DAMAGE || duel.is_damage_calculated() {
        return false;
    }
    let (attacker, defender) = (duel.get_attacker(), duel.get_attack_target());
    // Lua errors on a nil attacker
    unported!(duel, attacker.is_none());
    let Some(attacker) = attacker else { return false };
    let faceup_psychic = |c: Option<CardId>, duel: &Duel| {
        c.is_some_and(|c| c.is_faceup(duel) && c.is_race(duel, RACE_PSYCHIC) && c.is_relate_to_battle(duel))
    };
    let faceup_related = |c: Option<CardId>, duel: &Duel| c.is_some_and(|c| c.is_faceup(duel) && c.is_relate_to_battle(duel));
    if attacker.is_controler(duel, tp) {
        e.set_label_object_group(duel, defender.map(Into::into));
        faceup_psychic(Some(attacker), duel) && faceup_related(defender, duel)
    } else {
        e.set_label_object_group(duel, Some(attacker.into()));
        faceup_psychic(defender, duel) && faceup_related(Some(attacker), duel)
    }
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let bc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    // Lua errors on a nil battle opponent
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.check_lp_cost(tp, 100)
            && e.get_handler(duel).get_flag_effect(duel, ID) == 0
            && (bc.is_attack_above(duel, 100) || bc.is_defense_above(duel, 100)));
    }
    let mut lp = duel.get_lp(tp) - 1;
    let mut alp = 100;
    let maxpay = bc.get_attack(duel).max(bc.get_defense(duel));
    lp = lp.min(maxpay);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    if lp >= 200 {
        let options = (1..=lp.min(500) as u64 / 100).map(|n| n * 100).collect::<Vec<_>>();
        alp = duel.announce_number(tp, &options)? as u32;
    }
    duel.pay_lp_cost(tp, alp)?;
    e.set_label(duel, -i64::from(alp));
    e.get_handler(duel).register_flag_effect(duel, ID, RESET_PHASE | PHASE_DAMAGE, 0, 1);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    // Lua errors on a nil label object
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(false) };
    if let Some(chkc) = chkc {
        return Ok(chkc == tc);
    }
    if chk == 0 {
        return Ok(tc.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(tc);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || !tc.is_controler(duel, 1 - tp) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, e.get_label(duel) as i32);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    tc.register_effect(duel, e2);
    Ok(())
}
