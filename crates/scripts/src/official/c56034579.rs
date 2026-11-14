// GMX Chairman Kimridge
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 56034579;

pub const LISTED_SERIES: &[u16] = &[SET_GMX];

const APPLIED_EXPERIMENT_55: u32 = 18795635;

pub const LISTED_NAMES: &[u32] = &[APPLIED_EXPERIMENT_55];
// GMX Researcher Selande (Normal Summoned) and GMX Partner Selandea to Special Summon it by a monster effect, GMX Applied
// Experiment #55 in the Deck and Graveyard, and Dinosaur Normal Monsters (Trakodon, Mammoth Graveyard) in the Graveyard to put back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (20049870, HAND),
    (2618725, HAND),
    (APPLIED_EXPERIMENT_55, DECK),
    (APPLIED_EXPERIMENT_55, GRAVE),
    (42348802, GRAVE),
    (40374923, GRAVE),
    (42348802, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.re.is_some_and(|re| re.is_monster_effect(duel)));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, APPLIED_EXPERIMENT_55) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
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
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        if c.is_relate_to_effect(duel, a.e) {
            duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_GMX]) || c.is_race(duel, RACE_DINOSAUR)) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 2, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty()
        && duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0
        && tg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), 1, None)
    {
        if duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None)
            == 2
        {
            duel.sort_decktop(tp, tp, 2)?;
        }
        if duel.is_existing_matching_card(CardId::is_attack_pos, tp, 0, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, CardId::is_attack_pos, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
