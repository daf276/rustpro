// Earthbound Prisoner Line Walker
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 19322865;

pub const LISTED_SERIES: &[u16] = &[SET_EARTHBOUND];
pub const LISTED_NAMES: &[u32] = &[71089030, 7473735];

// In the hand to Normal Summon and in the Graveyard, with Earthbound Prison and Harmonic Synchro Fusion in the Deck and Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (71089030, DECK),
    (71089030, GRAVE),
    (7473735, DECK),
    (7473735, GRAVE),
];
// The Graveyard effect needs an Earthbound monster of Level 6 or more, none of which is ported, and an opponent's Extra Deck monster.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, tdcon);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, 71089030) || c.is_code(duel, 7473735)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn tdcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_EARTHBOUND]) && c.is_level_above(duel, 6)
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(tdcfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_summon_location(duel, LOCATION_EXTRA) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    c.is_code(duel, tc.get_code(duel))
        && c.is_can_be_special_summoned(duel, e, 0, 1 - tp, false, false)
        && duel.get_location_count_from_ex(1 - tp, 1 - tp, None, c) > 0
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
    {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, tc), tp, 0, LOCATION_EXTRA, None);
        if !g.is_empty() && duel.select_yes_no(1 - tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sc = g.select(duel, 1 - tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.special_summon(sc, 0, 1 - tp, 1 - tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
