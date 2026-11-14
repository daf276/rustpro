// Heraldic Beast Leo
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 82293134;

pub const LISTED_SERIES: &[u16] = &[SET_HERALDIC_BEAST];
// Normal Summoned from the hand, with Heraldic Beast Eale and Basilisk in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (82315772, DECK), (82315772, DECK), (56921677, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_normal_summoned(duel) && c.get_turn_id(duel) == duel.get_turn_count()
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERALDIC_BEAST]) && c.get_code(duel) != ID && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
