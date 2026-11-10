// Eye of Illusion
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 23446369;

// Artmage Power Patron, an Illusion, on the field, with the opponent's monsters to take control of and a Set copy for the opponent's
// turn. No scenario of 64 reached the attack redirect.
pub const SUPPORT: &[(u32, u16)] = &[
    (23829452, MZONE),
    (23829452, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (ID, SZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER);
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn control_filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_controler_can_be_changed(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ak = duel.get_attacker();
    if let Some(chkc) = chkc {
        let label = e.get_label(duel);
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_faceup(duel)
            && (label == 2 && chkc.is_controler_can_be_changed(duel) || label == 3 && Some(chkc) != ak));
    }
    let b1 = duel.get_flag_effect(tp, ID) == 0;
    let b2 = duel.is_turn_player(1 - tp) && duel.is_existing_target(control_filter, tp, 0, LOCATION_MZONE, 1, None);
    let b3 = duel.check_event(EVENT_ATTACK_ANNOUNCE)
        && ak.is_some_and(|ak| ak.is_controler(duel, 1 - tp))
        && duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, ak.map(CardSet::from));
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, 0);
        e.set_property(duel, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_CONTROL);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let g = duel.select_target(tp, control_filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    } else {
        e.set_category(duel, 0);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, ak.map(CardSet::from))?;
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        if duel.get_flag_effect(tp, ID) > 0 {
            return Ok(());
        }
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER));
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    } else if op == 2 {
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
        {
            duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
        }
    } else if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && let Some(ak) = duel.get_attacker()
        && ak.can_attack(duel)
        && ak.is_relate_to_battle(duel)
        && !ak.is_immune_to_effect(duel, e)
    {
        duel.calculate_damage(ak, Some(tc))?;
    }
    Ok(())
}
