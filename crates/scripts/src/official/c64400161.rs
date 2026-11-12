// Fandora, the Flying Furtress
use crate::keys::{REPLACE_COUNT, REPLACE_MAX};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 64400161;

// In the hand with Fur Hire monsters in the Deck to search and five different ones on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (ID, 0),
    (ID, 0),
    (ID, 0),
    (ID, 0),
    (ID, 0),
    (ID, HAND),
    (28004531, DECK),
    (93850652, DECK),
    (28004531, MZONE),
    (93850652, MZONE),
    (8728498, MZONE),
    (66736884, MZONE),
    (66023650, MZONE),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PREDRAW);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_condition(duel, descon);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.tp == duel.get_turn_player() && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 0 && duel.get_draw_count(a.tp) > 0
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let dt = duel.get_draw_count(tp);
    if dt == 0 {
        return Ok(());
    }
    duel.set_script_state(REPLACE_COUNT, 1);
    duel.set_script_state(REPLACE_MAX, i64::from(dt));
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_DRAW_COUNT);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_DRAW);
    e1.set_value(duel, 0);
    duel.register_effect(e1, tp);
    if duel.get_script_state(REPLACE_COUNT) > duel.get_script_state(REPLACE_MAX) || !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE])
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(cfilter, a.tp, LOCATION_MZONE, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 5
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(g, REASON_EFFECT)?;
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, e.get_handler(duel), 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}
