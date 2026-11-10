// Red-Eyes Metal Claws Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, SZONE};

const ID: u32 = 83711531;

pub const LISTED_NAMES: &[u32] = &[CARD_MAX_METALMORPH];

// In the hand with Set Traps (Trap Hole) to send as the cost, and Max Metalmorph on the field and in the Graveyard for the search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (4206964, SZONE),
    (4206964, SZONE),
    (CARD_MAX_METALMORPH, GRAVE),
    (CARD_MAX_METALMORPH, DECK),
    (CARD_MAX_METALMORPH, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_SZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_METALMORPH]));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn spcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_facedown(duel) && c.is_able_to_grave_as_cost(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spcostfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tc = duel.select_matching_card(tp, |c, duel| spcostfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.send_to_grave(tc, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_METALMORPH]) && c.is_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let max_metalmorph = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_MAX_METALMORPH));
        return Ok(
            duel.is_existing_matching_card(max_metalmorph, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
                && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
