// Foolish Graverobber
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT};

const ID: u32 = 65118318;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_TIME_WIZARD, ID];

// In the hand and Graveyard, with Dark Time Wizard in the Deck to send and in the Graveyard to Set, and a monster in each Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (40235813, DECK),
    (40235813, DECK),
    (40235813, GRAVE),
    (4881365, DECK),
    (1784619, GRAVE),
    (1784619, GRAVE | OPPONENT),
    (12580477, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(
        duel,
        TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    e2.set_hint_timing(
        duel,
        TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_DARK_TIME_WIZARD) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_EITHER, LOCATION_GRAVE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    let spfilter = aux::necro_valley_filter(|c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    if let Some(sc) = sc
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(&spfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, &spfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_spell_trap(duel)
        && c.is_ssetable(duel)
        && ((c.lists_code(duel, CARD_DARK_TIME_WIZARD) && !c.is_code(duel, ID)) || c.is_controler(duel, opp))
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| setfilter(c, duel, 1 - tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SET, None, 1, PLAYER_EITHER, LOCATION_GRAVE as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(
        tp,
        |c, duel| setfilter(c, duel, 1 - tp),
        tp,
        LOCATION_GRAVE,
        LOCATION_GRAVE,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.sset(tp, g)?;
    }
    Ok(())
}
