// Centur-Ion True Awakening
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 77543769;

// Centur-Ion Trudea and Centur-Ion Bonds to place Centur-Ion monsters as Continuous Traps to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (42493140, MZONE),
    (42493140, MZONE),
    (15005145, DECK),
    (15005145, DECK),
    (4160316, HAND),
    (42493140, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
            && duel.is_chain_negatable(a.ev as u8)
    });
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_original_type(duel, TYPE_MONSTER) && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cfilter, LOCATION_STZONE)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
