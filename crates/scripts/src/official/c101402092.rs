// Angelechy Strategy Score
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 101402092;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];
// In the hand with Angelechy Problem to search, and in the Graveyard with Problem in hand to place Angelechy Destrier from the
// Extra Deck when it summons Angelechy Enlisted.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (17782288, DECK),
    (17782288, HAND),
    (81797573, EXTRA),
    (55393975, EXTRA),
    (55393975, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, super::c17782288::tdtg);
    e2.set_operation(duel, |duel, a| super::c17782288::return_to_extra(duel, a, aux::stringid(ID, 3)));
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    register_extra_restriction(duel, e, tp, aux::stringid(ID, 2));
    Ok(())
}

pub fn register_extra_restriction(duel: &mut Duel, e: EffectId, tp: u8, desc: u64) {
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, desc);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_synchro_monster(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
}
