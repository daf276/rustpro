// Geomathmech Final Sigma
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 42632209;

pub const LISTED_SERIES: &[u16] = &[SET_MATHMECH];
// In the Extra Deck, Synchro Summoned with Geomathmech Magma (a Level 8 Tuner) and Level 4 Normal Monsters, with Mathmech cards in the
// Deck to search and the opponent's Raigeki and Dark Hole to destroy it, with copies on the field since the
// optional trigger is often declined.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (15248594, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (17946349, DECK),
    (17946349, DECK),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_EMZONE);
    e1.set_value(duel, Value::Effect(|duel, _, te| !te.is_card_setcode(duel, SET_MATHMECH)));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_controler(duel, 1 - c.get_controler(duel)))
    });
    e2.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (c.is_reason(duel, REASON_EFFECT) && a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MATHMECH]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
