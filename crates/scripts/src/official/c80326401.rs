// Wishes for Eyes of Blue
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 80326401;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON, ID];
// A copy in the hand and one in the Graveyard, Lunar Rabbit Omajinai (Level 1 LIGHT Tuner) and Chaos Form and Chaos Space (Spells
// that mention Blue-Eyes White Dragon) in the Deck, and Blue-Eyes White Dragon on the field with Blue-Eyes Ultimate Dragon to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (47643326, DECK),
    (47643326, DECK),
    (21082832, DECK),
    (99266988, DECK),
    (89631139, MZONE),
    (23995346, EXTRA),
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
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_level(duel, &[1]) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_type(duel, TYPE_TUNER))
        || (c.is_spell_trap(duel) && c.lists_code(duel, CARD_BLUEEYES_W_DRAGON) && !c.is_code(duel, ID)))
        && c.is_able_to_hand(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.filter_count(duel, CardId::is_monster, None) == 1, false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() < 2 {
        return Ok(());
    }
    let rg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    if !rg.is_empty() {
        duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, rg)?;
    }
    Ok(())
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLUE_EYES]) && c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_BLUEEYES_W_DRAGON));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_code(duel, CARD_BLUEEYES_W_DRAGON)
            && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(&filter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| eqfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, &filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(tp, |c, duel| eqfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(ec) = g.get_first()
        && duel.equip(tp, ec, tc)?
    {
        let e1 = duel.create_effect(ec);
        e1.set_type(duel, EFFECT_TYPE_EQUIP);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 400);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        ec.register_effect(duel, e1);
        let e2 = duel.create_effect(ec);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_EQUIP_LIMIT);
        e2.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        ec.register_effect(duel, e2);
    }
    Ok(())
}
