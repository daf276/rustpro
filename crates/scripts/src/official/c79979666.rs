// Elemental HERO Bubbleman
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 79979666;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
// The draw needs the player's hand and field empty, which no SUPPORT can arrange against the opening hand.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1)))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_field_group_count(tp, LOCATION_HAND, 0) == 1 && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_status(duel, STATUS_LEAVE_CONFIRMED)
}

fn others(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        filter,
        a.tp,
        LOCATION_ONFIELD | LOCATION_HAND,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !others(duel, a)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2));
    }
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if others(duel, a) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
