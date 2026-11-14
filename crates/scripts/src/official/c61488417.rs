// Chiwen, Light of the Yang Zing
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 61488417;

pub const LISTED_SERIES: &[u16] = &[SET_YANG_ZING];

const JIAOTU: u32 = 25935625;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field and in the Graveyard, with Jiaotu in the Deck to Special Summon and on the field to be destroyed beside the
// Graveyard copy, and Dark Hole in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (JIAOTU, DECK), (JIAOTU, MZONE), (53129443, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

pub(super) fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_controler(duel, a.tp)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YANG_ZING]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YANG_ZING])
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
