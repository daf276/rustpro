// Magical Musketeer Kidbrave
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 5230799;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];

const DESPERADO: u32 = 20745268;

// On the field in several columns, with Magical Musket - Desperado in the hand to activate in one of them, and cards in the Deck to draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (24154052, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, drcon);
    e3.set_cost(duel, drcost);
    e3.set_target(duel, stock::draw_target::<2>);
    e3.set_operation(duel, stock::draw);
    c.register_effect(duel, e3);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if duel.get_current_phase() & (PHASE_DAMAGE | PHASE_DAMAGE_CAL) > 0 || !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return false;
    }
    let p = duel.get_chain_info::<u8>(a.ev as u8, CHAININFO_TRIGGERING_CONTROLER);
    let seq = duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_SEQUENCE);
    a.e.get_handler(duel).is_column(duel, seq, p, LOCATION_SZONE)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGICAL_MUSKET]) && c.is_discardable(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}
