// Fire King High Avatar Garunix
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 23015896;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_KING];
// One on the field, destroyed by Raigeki to return in the Standby Phase, and Sacred Fire King Garunix in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (66431519, DECK),
    (66431519, DECK),
    (12580477, DECK | OPPONENT),
    (39674352, MZONE | OPPONENT),
    (32626733, MZONE | OPPONENT),
    (56649609, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_operation(duel, spreg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BATTLE_DESTROYED);
    e4.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
    });
    e4.set_target(duel, sptg2);
    e4.set_operation(duel, spop2);
    c.register_effect(duel, e4);
}

fn spreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if a.r & (REASON_DESTROY | REASON_EFFECT) != (REASON_DESTROY | REASON_EFFECT) {
        return Ok(());
    }
    if duel.is_phase(PHASE_STANDBY) {
        let turn = i64::from(duel.get_turn_count());
        e.set_label(duel, turn);
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 0, 2);
    } else {
        e.set_label(duel, 0);
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 0, 1);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel)
        .is_some_and(|e1| e1.get_label(duel) != i64::from(duel.get_turn_count()))
        && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(a.e.get_handler(duel).into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(a.e.get_handler(duel).into()));
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FIRE_KING]) && c.get_code(duel) != ID && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
