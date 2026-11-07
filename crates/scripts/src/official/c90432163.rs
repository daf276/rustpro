// The Phantom Knights of Ancient Cloak
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 90432163;
pub const LISTED_NAMES: &[u32] = &[ID];

// A second copy on the field and in the GY, and Doomed Soleret in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (83566725, DECK), (83566725, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_position(duel, POS_FACEUP_ATTACK));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_DARK)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) || duel.change_position(c, POS_FACEUP_DEFENSE)? == 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 800);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
