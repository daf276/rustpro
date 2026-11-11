// Ally of Justice Cyclone Creator
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 45586855;

// On the field with Lunar Rabbit Omajinai (a Tuner) and Level Limit - Area B (a Continuous Spell) to return, and Mokey Mokey in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (47643326, MZONE),
    (47643326, MZONE),
    (3136426, SZONE),
    (3136426, SZONE),
    (3136426, SZONE),
    (27288416, HAND),
    (27288416, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_TUNER)
}

fn rfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, ct, None));
    }
    let rg = duel.get_matching_group(rfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(rg), ct as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let rg = duel.get_matching_group(rfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if rg.len() < ct {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let sg = rg.select(duel, tp, ct as u8, ct as u8, None)?;
    duel.hint_selection(sg);
    duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
