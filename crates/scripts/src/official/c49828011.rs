// Officiating Reverie
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 49828011;

// In the hand and the Graveyard, and banished; Wollow (a Zombie Xyz Monster, also the Zombie to Special Summon) on the field,
// destroyed by the opponent's Smashing Ground and Fissure.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, REMOVED),
    (45935145, MZONE),
    (45935145, MZONE),
    (97169186, HAND | OPPONENT),
    (66788016, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, |duel, a, chk| Cost::discard_full(duel, a, chk, aux::TRUE, true, (1, 1)));
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, gyspcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, gyspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e3.set_range(duel, LOCATION_REMOVED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| duel.get_turn_count() == a.e.get_handler(duel).get_turn_id(duel) + 1);
    e3.set_target(duel, attachtg);
    e3.set_operation(duel, attachop);
    c.register_effect(duel, e3);
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn gyspcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, CardId::is_xyz_monster, 1, None)
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| gyspfilter(c, duel, e, tp),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            Some(e.get_handler(duel).into()),
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8, hc: CardId) -> bool {
    c.is_race(duel, RACE_ZOMBIE)
        && c.is_xyz_monster(duel)
        && c.is_faceup(duel)
        && hc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel, tp, c));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| xyzfilter(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |tc, duel| xyzfilter(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.overlay(tc, c)?;
    }
    Ok(())
}
