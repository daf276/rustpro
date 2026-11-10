// Turbo Synchron
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 67270095;

// Two on the field attacking Luster Dragon, with Psychic Kappa, a Normal Monster, in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (11091375, MZONE | OPPONENT),
    (11091375, MZONE | OPPONENT),
    (11091375, MZONE | OPPONENT),
    (11091375, MZONE | OPPONENT),
    (7892180, HAND),
    (7892180, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn postg(duel: &mut Duel, _: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let d = duel.get_attack_target();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == d);
    }
    if chk == 0 {
        return Ok(d.is_some_and(|d| d.is_attack_pos(duel) && d.is_can_change_position(duel)));
    }
    if let Some(d) = d {
        duel.set_target_card(d);
        duel.set_operation_info(0, CATEGORY_POSITION, Some(d.into()), 1, 0, 0);
    }
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_attack_pos(duel)
    {
        duel.change_position(tc, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.r == REASON_BATTLE && a.ep == a.tp && duel.get_attacker() == Some(a.e.get_handler(duel))
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, dam: i32) -> bool {
    c.is_attack_below(duel, dam) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, ev) = (a.e, a.tp, a.ev as i32);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp, ev), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, ev) = (a.e, a.tp, a.ev as i32);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp, ev), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
