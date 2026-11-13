// Candelato, the Beast of Light
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 8315896;

// In the hand and on the field, with Pot of Greed to discard and Witchcrafter Marka to activate a search in response, and Jirai Gumo
// and another card with a coin tossing effect in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (55144522, HAND),
    (55144522, HAND),
    (6071005, HAND),
    (94773007, DECK),
    (81480460, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_value(
        duel,
        Value::Card(|duel, _, c| 1000 * duel.get_field_group_count(c.get_controler(duel), LOCATION_HAND, 0) as i32),
    );
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, |duel, a, chk| Cost::discard_full(duel, a, chk, aux::TRUE, true, (2, 2)));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, discon);
    e3.set_target(duel, distg);
    e3.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    super::toss_coin(c.get_original_code(duel)) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    if !duel.is_chain_disablable(ev) {
        return false;
    }
    if a.re
        .is_some_and(|re| re.is_has_category(duel, CATEGORY_SEARCH) || re.is_has_category(duel, CATEGORY_DRAW))
    {
        return true;
    }
    let opinfo = duel.get_operation_info(ev, CATEGORY_TOHAND);
    let popinfo = duel.get_possible_operation_info(ev, CATEGORY_TOHAND);
    if opinfo.is_none() && popinfo.is_none() {
        return false;
    }
    let g = opinfo
        .and_then(|info| info.op_cards)
        .unwrap_or_default()
        .union(popinfo.and_then(|info| info.op_cards).unwrap_or_default());
    let locations = opinfo.map_or(0, |info| info.op_param as u32) | popinfo.map_or(0, |info| info.op_param as u32);
    locations & LOCATION_DECK > 0 || (!g.is_empty() && g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
    Ok(false)
}
