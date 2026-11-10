// Cocoon of Evolution
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 40240595;
const PETIT_MOTH: u32 = 58192742;

pub const LISTED_NAMES: &[u32] = &[PETIT_MOTH, ID];

// Copies in the hand to equip Petit Moths (Normal Monsters) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (PETIT_MOTH, MZONE),
    (PETIT_MOTH, MZONE),
    (PETIT_MOTH, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_condition(duel, checkcon);
    e2.set_operation(duel, checkop);
    e2.set_count_limit(duel, 1, 0);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EVENT_ADJUST);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_condition(duel, checkcon2);
    e3.set_operation(duel, checkop2);
    c.register_effect(duel, e3);
}

fn checkcon(duel: &Duel, a: &Args) -> bool {
    let g = a.e.get_handler(duel).get_card_target(duel);
    duel.is_turn_player(a.tp) && g.is_exists(duel, |c, duel| c.is_code(duel, PETIT_MOTH), 1, None)
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = c.get_turn_counter(duel) + 1;
    c.set_turn_counter(duel, ct);
    Ok(())
}

fn checkcon2(duel: &Duel, a: &Args) -> bool {
    let g = a.e.get_handler(duel).get_card_target(duel);
    !g.is_empty() && g != a.e.get_label_object_group(duel).unwrap_or_default()
}

fn checkop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let g = c.get_card_target(duel);
    let g2 = e.get_label_object_group(duel);
    let new_targets = g.filter(duel, aux::TRUE, g2);
    if !g.is_exists(duel, |c, duel| c.is_code(duel, PETIT_MOTH), 1, Some(new_targets)) {
        c.set_turn_counter(duel, 0);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, PETIT_MOTH)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    let Some(tc) = tc.filter(|tc| {
        duel.get_location_count(tp, LOCATION_SZONE) > 0
            && !tc.is_controler(duel, 1 - tp)
            && !tc.is_facedown(duel)
            && tc.is_relate_to_effect(duel, e)
    }) else {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    };
    duel.equip(tp, c, tc)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e1.set_label_object_group(duel, Some(tc.into()));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e2.set_value(duel, 0);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_SET_BASE_DEFENSE);
    e3.set_value(duel, 2000);
    c.register_effect(duel, e3);
    Ok(())
}
