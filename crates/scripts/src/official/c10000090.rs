// The Winged Dragon of Ra - Immortal Phoenix
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 10000090;
const SPHERE_MODE: u32 = 10000080;

pub const LISTED_NAMES: &[u32] = &[CARD_RA, SPHERE_MODE];
// In the Graveyard to return when The Winged Dragon of Ra is destroyed by Dark Hole, and on the field to pay LP for sending a monster; The Winged Dragon
// of Ra - Sphere Mode in the Deck and Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, MZONE),
    (CARD_RA, MZONE),
    (SPHERE_MODE, DECK),
    (SPHERE_MODE, GRAVE),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, spcon1);
    e2.set_target(duel, sptg1);
    e2.set_operation(duel, spop1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, Cost::pay_lp::<1000>);
    e4.set_target(duel, tgtg);
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_code(duel, EVENT_PHASE + PHASE_END);
    e5.set_target(duel, sptg2);
    e5.set_operation(duel, spop2);
    c.register_effect(duel, e5);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CARD_RA) && c.is_controler(duel, tp) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_chain_limit(|_, _, _, _, _| false, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, true, true, POS_FACEUP)? != 0 {
        c.complete_procedure(duel);
    }
    Ok(())
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, SPHERE_MODE) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        a.tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.send_to_grave(c, REASON_EFFECT)? != 0 && c.is_location(duel, LOCATION_GRAVE) {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
