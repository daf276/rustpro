// Castle of Dark Illusions
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 62121;

// Face-down on the field to be flipped by an attack from the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    let fid = g
        .get_max_group(duel, |c, duel| c.get_field_id(duel).into())
        .map_or(0, |(_, fid)| fid);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| {
        i64::from(c.get_field_id(duel)) <= e.get_label(duel) && c.is_race(duel, RACE_ZOMBIE)
    });
    e1.set_value(duel, 200);
    e1.set_label(duel, fid);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_operation(duel, atkop);
    e3.set_label_object(duel, Some(e2));
    e3.set_label(duel, 2);
    e3.set_reset_full(
        duel,
        RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
        4,
    );
    c.register_effect(duel, e3);
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ct = e.get_label(duel);
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_value(duel, (ct * 200) as i32);
        if let Some(e1) = e2.get_label_object(duel) {
            e1.set_value(duel, (ct * 200) as i32);
        }
    }
    e.set_label(duel, ct + 1);
    Ok(())
}
