// Tidal, Dragon Ruler of Waterfalls
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 26400609;

// Copies in the hand, the Graveyard and the Deck (to be banished as a cost, then searched), with Hyosube (WATER) and Red-Eyes Black
// Dragon (Dragon) as the cost cards and the Deck monster to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, DECK),
    (ID, DECK),
    (2118022, HAND),
    (74677422, HAND),
    (74677422, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_and_return_effects::<ID, ATTRIBUTE_WATER>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_WATER>);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
    add_banished_search_effect::<ID, ATTRIBUTE_WATER>(duel, c);
}

// The effects the Level 7 Dragon Rulers of the four attributes share: the Special Summon from the hand or Graveyard and the
// return to the hand, then (after the effect of their own) the search when banished.
pub fn add_summon_and_return_effects<const ID: u32, const ATTRIBUTE: u32>(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, selfspcost::<ATTRIBUTE>);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE | PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && a.e.get_handler(duel).is_special_summoned(duel)
    });
    e2.set_target(duel, rthtg);
    e2.set_operation(duel, rthop);
    c.register_effect(duel, e2);
}

pub fn add_banished_search_effect<const ID: u32, const ATTRIBUTE: u32>(duel: &mut Duel, c: CardId) {
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_REMOVE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg::<ATTRIBUTE>);
    e4.set_operation(duel, thop::<ATTRIBUTE>);
    c.register_effect(duel, e4);
}

fn spcostfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel) -> bool {
    (c.is_race(duel, RACE_DRAGON) || c.is_attribute(duel, ATTRIBUTE))
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
}

fn selfspcost<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let rg = duel.get_matching_group(
        spcostfilter::<ATTRIBUTE>,
        tp,
        LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE,
        0,
        Some(c.into()),
    );
    if chk == 0 {
        return Ok(rg.len() > 1 && aux::select_unselect_group_check(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1)));
    }
    let g = aux::select_unselect_group(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(a.e.get_handler(duel).into()), 1, a.tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn thfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_attribute(duel, ATTRIBUTE) && c.is_able_to_hand(duel)
}

fn thtg<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter::<ATTRIBUTE>, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter::<ATTRIBUTE>, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
