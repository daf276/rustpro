// Infernoble Knight Ogier
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 21351206;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_NOBLE_ARMS];

// In hand to be summoned with Infernoble Arms - Durendal in the Deck to send, and in the Graveyard with Buster Blader to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (37478723, DECK),
    (37478723, DECK),
    (CARD_BUSTER_BLADER, MZONE),
    (CARD_BUSTER_BLADER, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_category(duel, CATEGORY_EQUIP);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, eqtg);
    e3.set_operation(duel, eqop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_EQUIP);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_FIRE)) || c.is_set_card(duel, &[SET_NOBLE_ARMS]))
        && c.is_able_to_grave(duel)
        && !c.is_code(duel, ID)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

pub fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_WARRIOR)
}

pub fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

pub fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && tc.is_relate_to_effect(duel, e)
        && tc.is_controler(duel, tp)
        && !tc.is_facedown(duel)
    {
        duel.equip_full(tp, c, tc, true, false)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        c.register_effect(duel, e1);
    }
    Ok(())
}
