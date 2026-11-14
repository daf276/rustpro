// Ido the Supreme Magical Force
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 35984222;

// On the field for the opponent's Raigeki and Dark Hole to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_operation(duel, spr);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.get_turn_id(duel) != duel.get_turn_count() && c.get_flag_effect(duel, ID) > 0
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 0);
    c.register_effect(duel, e3);
    for code in [EFFECT_CANNOT_FLIP_SUMMON, EFFECT_CANNOT_SUMMON, EFFECT_CANNOT_MSET] {
        let e = e3.clone_effect(duel);
        e.set_code(duel, code);
        c.register_effect(duel, e);
    }
}

fn spr(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.r & (REASON_DESTROY | REASON_EFFECT) != (REASON_DESTROY | REASON_EFFECT) || c.is_previous_location(duel, LOCATION_SZONE) {
        return Ok(());
    }
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 2);
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, 0, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 1, a.tp, a.tp, false, false, POS_FACEUP)? != 0 {
        let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, 0, Some(c.into()));
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
