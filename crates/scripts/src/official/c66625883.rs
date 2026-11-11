// Karakuri Strategist mdl 248 "Nishipachi"
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 66625883;

// Copies on the field and in the hand, with a monster of the opponent's to attack it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, HAND), (ID, HAND), (27125110, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MUST_ATTACK);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_POSITION);
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e3.set_condition(duel, poscon);
    e3.set_operation(duel, posop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_POSITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EVENT_SUMMON_SUCCESS);
    e4.set_target(duel, tg);
    e4.set_operation(duel, op);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e5);
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_attack_pos(duel)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_can_change_position(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}
