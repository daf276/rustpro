// Dark Flare Swordsman
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 94979322;
pub const LISTED_NAMES: &[u32] = &[CARD_FLAME_SWORDSMAN];

// Copies in the hand and on the field, Flame Swordsman in the Extra Deck to send, and Fighting Flame Swordsman in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_FLAME_SWORDSMAN, EXTRA),
    (1047075, DECK),
    (1047075, DECK),
    (1047075, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, selfspcost);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_tribute);
    e3.set_target(duel, decksptg);
    e3.set_operation(duel, deckspop);
    c.register_effect(duel, e3);
}

fn selfcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_FLAME_SWORDSMAN) || c.lists_code(duel, CARD_FLAME_SWORDSMAN)) && c.is_able_to_grave_as_cost(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, selfcostfilter, LOCATION_EXTRA)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_WARRIOR));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.lists_code(duel, CARD_FLAME_SWORDSMAN) && !c.is_level(duel, &[6]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn decksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| deckspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn deckspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| deckspfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
