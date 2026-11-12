// Procession of the Tea Jar
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 12612470;

// On the field with monsters on both sides to share its column with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (3573512, MZONE | OPPONENT),
    (3573512, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, coltg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, _| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && coltg(duel, e, re.get_handler(duel))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, aux::seqmovcon);
    e3.set_operation(duel, aux::seqmovop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_MOVE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, poscon);
    e4.set_target(duel, postg);
    e4.set_operation(duel, posop);
    c.register_effect(duel, e4);
}

fn coltg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_facedown(duel) && e.get_handler(duel).get_column_group(duel).contains(c)
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_MZONE) && c.is_previous_location(duel, LOCATION_MZONE)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_can_turn_set(duel) && c.is_location(duel, LOCATION_MZONE)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = a.e.get_handler(duel).get_column_group(duel).filter(duel, posfilter, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, a.tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = c.get_column_group(duel).filter(duel, posfilter, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
