// Cyberdark Chimera
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 5370235;
const POWER_BOND: u32 = 37630732;

pub const LISTED_NAMES: &[u32] = &[POWER_BOND];
pub const LISTED_SERIES: &[u16] = &[SET_CYBER, SET_CYBERDARK];

// On the field with Spells in hand to discard and Power Bond in the Deck; Cyberdark monsters in the Deck to send when the
// opponent's Raigeki destroys it; Cyber Dragons and Cyber Twin Dragon to Fusion Summon under its material limit.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, HAND),
    (74157028, EXTRA),
    (POWER_BOND, HAND),
    (POWER_BOND, DECK),
    (POWER_BOND, DECK),
    (82562802, DECK),
    (41230939, DECK),
    (56100345, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(74157028),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn discardable_spell_trap(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_discardable(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(discardable_spell_trap, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, discardable_spell_trap, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, POWER_BOND) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    stock::search(duel, a, thfilter, LOCATION_DECK)?;
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_FUSION_MATERIAL);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target_range(duel, LOCATION_GRAVE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_able_to_remove(duel) && c.is_monster(duel));
    e1.set_fusion_material_operation(duel, Fusion::banish_material);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_target_range(duel, 0xff, 0xff);
    e2.set_target_card(duel, |duel, _, c| {
        !(c.is_set_card(duel, &[SET_CYBER]) && c.is_race(duel, RACE_DRAGON | RACE_MACHINE))
    });
    e2.set_value(
        duel,
        Value::OptionalCard(|duel, e, c| i32::from(c.is_some_and(|c| c.is_controler(duel, e.get_handler_player(duel))))),
    );
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let code = c.get_code(duel);
    c.is_set_card(duel, &[SET_CYBERDARK])
        && c.is_monster(duel)
        && c.is_able_to_grave(duel)
        && !duel.is_existing_matching_card(|gc, duel| gc.is_code(duel, code), tp, LOCATION_GRAVE, 0, 1, None)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    stock::exists_target(duel, a, chk, |c, duel| tgfilter(c, duel, tp), LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    stock::send_to_grave(duel, a, |c, duel| tgfilter(c, duel, tp), LOCATION_DECK)
}
