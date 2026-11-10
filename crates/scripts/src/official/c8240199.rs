// Sage with Eyes of Blue
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 8240199;

// A copy in the hand to Normal Summon or discard, Lunar Rabbit Omajinai (Level 1 LIGHT Tuner) and Blue-Eyes White Dragon in
// the Deck, and Sangan to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (47643326, DECK),
    (47643326, DECK),
    (89631139, DECK),
    (89631139, DECK),
    (26202165, MZONE),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(89631139)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_discard);
    e2.set_target(duel, gvtg);
    e2.set_operation(duel, gvop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.get_level(duel) == 1
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
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

fn gvfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_able_to_grave(duel) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c, duel: &Duel| gvfilter(c, duel, ft);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(ft > -1
            && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn gvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_grave(tc, REASON_EFFECT)? != 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
