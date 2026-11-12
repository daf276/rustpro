// Ha-Re the Sword Mikanko
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 18377261;

// On the field with Mage Power in the hand to equip it and the opponent's attackers. No Mikanko Equip Spell is ported, so the search
// has nothing to find.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (83746708, HAND),
    (83746708, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Message(MSG_ATTACK),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_condition(duel, |duel, a| !eqcon(duel, a));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, eqcon);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_EQUIP);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_equip_count(duel) > 0
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MIKANKO]) && c.is_type(duel, TYPE_EQUIP) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
