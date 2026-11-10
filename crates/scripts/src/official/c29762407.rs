// Temple of the Kings
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 29762407;
const SERKET: u32 = 89194033;

pub const LISTED_NAMES: &[u32] = &[SERKET];
// In the hand and Set with Mystical Beast of Serket on the field to send, and Special Summons (Skull Servant, Flame Ghost) to find.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (SERKET, MZONE),
    (SERKET, MZONE),
    (CARD_SKULL_SERVANT, DECK),
    (CARD_SKULL_SERVANT, HAND),
    (58528964, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_SZONE, 0);
    e2.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, cost);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rp: u8) -> bool {
    if c.is_facedown(duel) || !c.is_code(duel, SERKET) || !c.is_able_to_grave_as_cost(duel) {
        return false;
    }
    let sg = Some(CardSet::from_iter([c, e.get_handler(duel)]));
    duel.is_existing_matching_card(
        |sc, duel| filter(sc, duel, e, tp, rp, sg),
        tp,
        LOCATION_DECK | LOCATION_HAND | LOCATION_EXTRA,
        0,
        1,
        None,
    )
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    if chk == 0 {
        return Ok(e.get_handler(duel).is_able_to_grave_as_cost(duel)
            && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, e, tp, rp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let mut g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, e, tp, rp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    g.insert(e.get_handler(duel));
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rp: u8, sg: Option<CardSet>) -> bool {
    if !c.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
        return false;
    }
    if c.is_location(duel, LOCATION_HAND | LOCATION_DECK) {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    } else {
        c.is_type(duel, TYPE_FUSION) && duel.get_location_count_from_ex(tp, rp, sg, c) > 0
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let location = (LOCATION_DECK | LOCATION_HAND | LOCATION_EXTRA) as i32;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, location);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp, rp, None),
        tp,
        LOCATION_DECK | LOCATION_HAND | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
