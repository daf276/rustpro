// Void Vanishment
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 78748366;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_VOID, SET_INFERNOID];
// Face-up in the Spell & Trap Zone with Void Feast in the Deck to add, Infernoid Onuncu to battle the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (31548814, DECK),
    (31548814, DECK),
    (14799437, MZONE),
    (14799437, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

pub fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VOID]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_INFERNOID]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}

fn battlers(duel: &Duel, tp: u8) -> (Option<CardId>, Option<CardId>) {
    let (a, d) = (duel.get_attacker(), duel.get_attack_target());
    unported!(duel, a.is_none());
    if a.is_some_and(|a| a.is_controler(duel, 1 - tp)) {
        (d, a)
    } else {
        (a, d)
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (at, d) = battlers(duel, a.tp);
    let (Some(at), Some(d)) = (at, d) else {
        return Ok(false);
    };
    if chk == 0 {
        return Ok(at.is_set_card(duel, &[SET_INFERNOID]) && (at.is_able_to_remove(duel) || d.is_able_to_remove(duel)));
    }
    let g: CardSet = [at, d].into_iter().filter(|c| c.is_relate_to_battle(duel)).collect();
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let rg: CardSet = [duel.get_attacker(), duel.get_attack_target()]
        .into_iter()
        .flatten()
        .filter(|c| c.is_relate_to_battle(duel))
        .collect();
    duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
