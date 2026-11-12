// Fire Ant Ascator
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 78275321;

// On the field with Level 5 Normal Monsters (Shapesnatch, Orion the Battle King) in the Graveyard and an attacker on the
// opponent's side to destroy it in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (4035199, GRAVE),
    (2971090, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.get_level(duel) == 5 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e) && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e2, true);
        duel.special_summon_complete()?;
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e3.set_range(duel, LOCATION_MZONE);
        e3.set_code(duel, EVENT_PHASE + PHASE_END);
        e3.set_operation(duel, desop);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e3.set_count_limit(duel, 1, 0);
        tc.register_effect(duel, e3);
    }
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.send_to_grave(a.e.get_handler(duel), REASON_EFFECT)?;
    Ok(())
}
