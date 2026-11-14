// Adamancipator Crystal - Tiamite
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 51420096;

pub const LISTED_SERIES: &[u16] = &[SET_ADAMANCIPATOR];
pub const LISTED_NAMES: &[u32] = &[ID];
// Special Summoned from the Deck by the excavating Adamancipator monsters, with Adamancipator Mates to add and a Rock Synchro in the GY.
// Special Summoned from the Deck by the excavation of the Adamancipator monsters, with Adamancipator Mates in the Deck to add, and in the GY with a Rock Synchro Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (30068809, DECK),
    (30068809, DECK),
    (ID, GRAVE),
    (24925387, MZONE),
    (85914562, MZONE),
    (48519867, MZONE),
    (11302671, MZONE),
    (47674738, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_card_setcode(duel, SET_ADAMANCIPATOR) && re.is_has_type(duel, EFFECT_TYPE_ACTIONS))
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ADAMANCIPATOR]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty()
        || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0
        || !g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND))
    {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    duel.shuffle_deck(tp)?;
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let hg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    if hg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let hsg = hg.select(duel, tp, 1, 1, None)?;
    if !hsg.is_empty() {
        duel.break_effect()?;
        duel.special_summon(hsg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_synchro_monster(duel) && c.is_faceup(duel) && c.is_able_to_extra(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_MZONE | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, location) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel) && duel.is_existing_target(tdfilter, tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, location, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOEXTRA, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_EXTRA)
        && c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0
    {
        duel.confirm_decktop(tp, 1)?;
    }
    Ok(())
}
