// Gagaku-P.U.N.K. Wa Gon
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 50642380;

// Gagaku-P.U.N.K. Crash Beat in the Deck to search, and the opponent's Sangans to attack it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (17691568, DECK),
    (17691568, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::pay_lp::<600>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_BECOME_TARGET);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, efdrcon);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e3.set_condition(duel, btdrcon);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PUNK]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn drconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel) && c.is_set_card(duel, &[SET_PUNK])
}

fn efdrcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.rp == 1 - tp
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| drconfilter(c, duel, tp), 1, None))
}

fn btdrcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_attack_target()
        .is_some_and(|tc| duel.is_turn_player(1 - tp) && drconfilter(tc, duel, tp))
}
