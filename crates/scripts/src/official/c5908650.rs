// Shopina the Melodious Maestra
use super::c62895219::grave_to_hand_target;
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 5908650;

// On the field, with Aria and Elegy (LIGHT Fairies) in the GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (40502912, GRAVE), (79514956, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
}

fn chainfilter(duel: &Duel, re: EffectId, _: u8, cid: u8) -> bool {
    let attr: u32 = duel.get_chain_info(cid, CHAININFO_TRIGGERING_ATTRIBUTE);
    !(re.is_monster_effect(duel) && attr != ATTRIBUTE_LIGHT)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_CHAIN) == 0);
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && re.get_handler(duel).is_attribute_except(duel, ATTRIBUTE_LIGHT)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FAIRY) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    grave_to_hand_target(duel, a, chk, chkc, thfilter)
}
