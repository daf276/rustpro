// Limit Reverse
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 27551;

// In the hand to Activate, with monsters of 1000 ATK or less (Psychic Kappa, Change Slime) in the Graveyard, and the opponent's
// Mystical Space Typhoon, Dark Hole and monsters to destroy it, the target and change its position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (7892180, GRAVE),
    (18914778, GRAVE),
    (5318639, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (14851496, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE | TIMING_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_condition(duel, descon2);
    e3.set_operation(duel, desop2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_code(duel, EVENT_CHANGE_POS);
    e4.set_condition(duel, descon3);
    e4.set_operation(duel, desop3);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attack_below(duel, 1000) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
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

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?
    {
        c.set_card_target(duel, tc);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_first_card_target(duel)
        && tc.is_location(duel, LOCATION_MZONE)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon2(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.is_contains(tc)) && tc.is_reason(duel, REASON_DESTROY))
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.destroy(a.e.get_handler(duel), REASON_EFFECT)?;
    Ok(())
}

fn descon3(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.is_contains(tc)) && tc.is_defense_pos(duel))
}

fn desop3(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = c.get_first_card_target(duel) {
        duel.destroy(CardSet::from_iter([tc, c]), REASON_EFFECT)?;
    }
    Ok(())
}
