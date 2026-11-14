// Maliss <P> White Rabbit
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 69272449;

pub const LISTED_SERIES: &[u16] = &[SET_MALISS];
// With Maliss <C> MTP-07 in the Deck to Set, and Maliss <P> March Hare to banish it from the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (94722358, DECK), (94722358, DECK), (20938824, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_condition(duel, bdcon);
    e1.set_value(duel, 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_REMOVE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_cost(duel, Cost::pay_lp::<300>);
    e4.set_target(duel, super::c96676583::sptg);
    e4.set_operation(duel, |duel, a| super::c96676583::revive_and_limit(duel, a, aux::stringid(ID, 2)));
    c.register_effect(duel, e4);
}

fn linkfilter(c: Option<CardId>, duel: &Duel, hc: CardId) -> bool {
    c.is_some_and(|c| c.is_set_card(duel, &[SET_MALISS]) && c.is_type(duel, TYPE_LINK) && c.get_linked_group(duel).is_contains(hc))
}

fn bdcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    linkfilter(duel.get_attacker(), duel, c) || linkfilter(duel.get_attack_target(), duel, c)
}

fn setfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MALISS])
        && c.is_trap(duel)
        && c.is_ssetable(duel)
        && !duel.is_existing_matching_card(|g, duel| g.is_code(duel, c.get_code(duel)), tp, LOCATION_GRAVE, 0, 1, None)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk != 0 || duel.is_existing_matching_card(|c, duel| setfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, |c, duel| setfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
