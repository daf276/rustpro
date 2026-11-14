// Exosister Pax
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND, MZONE};

const ID: u32 = 77913594;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand, with Exosister Elis (which lists Stella) in the Deck to search and Stella in the Graveyard and on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (16474916, DECK),
    (16474916, DECK),
    (43863925, GRAVE),
    (43863925, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase_full(None));
    e1.set_cost(duel, Cost::pay_lp::<800>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn listfilter(c: CardId, duel: &Duel, sc: CardId) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_faceup(duel) && c.is_monster(duel) && sc.lists_code(duel, c.get_code(duel))
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? < 1 || !sc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, sc)?;
    duel.shuffle_hand(tp)?;
    if sc.is_monster(duel)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|c, duel| listfilter(c, duel, sc), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 0))?
    {
        duel.break_effect()?;
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
