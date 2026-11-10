// Burst Stream of Destruction
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 17655904;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// In the hand, with Blue-Eyes White Dragon on the field and the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_ATTACK,
        CounterFilter::Card(|duel, c| !c.is_code(duel, CARD_BLUEEYES_W_DRAGON)),
    );
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_BLUEEYES_W_DRAGON));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_ATTACK) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_property(duel, EFFECT_FLAG_OATH + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_target_card(duel, |duel, _, c| c.is_code(duel, CARD_BLUEEYES_W_DRAGON));
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    let sg = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let sg = duel.get_matching_group(aux::TRUE, a.tp, 0, LOCATION_MZONE, None);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
