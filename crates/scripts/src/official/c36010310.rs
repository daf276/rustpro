// Nunu, the Ogdoadic Remnant
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 36010310;

pub const LISTED_SERIES: &[u16] = &[SET_OGDOADIC];
// Copies in hand and the GY, Nauya to send from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (62405028, DECK), (62405028, DECK)];
// Both effects use description 0 in the Lua.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        || duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_OGDOADIC])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

// The banish-when-it-leaves redirect an Ogdoadic monster summoned from the GY gets.
pub fn banish_on_leave(duel: &mut Duel, owner: CardId, tc: CardId) {
    let e1 = duel.create_effect(owner);
    e1.set_description(duel, 3300);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    tc.register_effect_full(duel, e1, true);
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        banish_on_leave(duel, c, c);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_absolute_range(duel, tp, 1, 0);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e2.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_REPTILE));
        c.register_effect(duel, e2);
    }
    Ok(())
}
