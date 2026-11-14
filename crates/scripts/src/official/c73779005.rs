// Dragoons of Draconia
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT, PZONE};

const ID: u32 = 73779005;

// In the Pendulum Zone, with Battle Ox (Normal) to destroy Petit Moth (weak) in battle, and Uraby (Level 4 Normal) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (5053103, MZONE),
    (5053103, MZONE),
    (58192742, MZONE | OPPONENT),
    (58192742, MZONE | OPPONENT),
    (1784619, DECK),
    (1784619, DECK),
];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let (attacker, defender) = (duel.get_attacker(), duel.get_attack_target());
    let Some(mut d) = defender else { return false };
    unported!(duel, attacker.is_none());
    let Some(mut at) = attacker else { return false };
    if d.is_controler(duel, a.tp) {
        (at, d) = (d, at);
    }
    at.is_type(duel, TYPE_NORMAL) && !at.is_status(duel, STATUS_BATTLE_DESTROYED) && d.is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_level_above(duel, 4) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
