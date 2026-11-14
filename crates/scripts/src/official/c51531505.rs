// Dragonpit Magician
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, PZONE, SZONE};

const ID: u32 = 51531505;

// In the Pendulum Zone with another Magician (Dragonpulse Magician) and Pendulum Monsters in the hand to discard, and the opponent's
// Spells and Traps (Deck Lockdown) to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (15146890, PZONE),
    (15146890, HAND),
    (15146890, HAND),
    (1149109, SZONE | OPPONENT),
    (1149109, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

pub fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_set_card(duel, &[SET_MAGICIAN]),
        a.tp,
        LOCATION_PZONE,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && c.is_discardable(duel)
}

pub fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

pub fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
