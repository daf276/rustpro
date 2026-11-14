// Mimighoul Dungeon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 86809440;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// In the hand to activate, with Mimighoul Fairy in the Deck and Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (43066927, DECK), (43066927, DECK), (43066927, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_MIMIGHOUL]) && !c.is_status(duel, STATUS_SUMMON_TURN | STATUS_SPSUMMON_TURN)
    });
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_base_defense(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_SUMMON);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_target_spsummon(duel, |duel, _, _, sump, _, _, _, _| {
        duel.is_existing_matching_card(CardId::is_facedown, sump, LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, _, c| {
        c.is_status(duel, STATUS_SPSUMMON_TURN)
            && duel.is_existing_matching_card(CardId::is_facedown, c.get_controler(duel), LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MIMIGHOUL]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_DECK | LOCATION_GRAVE)
}
