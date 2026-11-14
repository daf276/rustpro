// Ryu-Ge War Zone
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, REMOVED};

const ID: u32 = 55276522;
const MISTVA: u32 = 92487128;

pub const LISTED_NAMES: &[u32] = &[MISTVA];
pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
// In the hand to activate, with a Ryu-Ge Dinosaur, Sea Serpent and Wyrm (Kaiva, Emva, Hakva) each in the Deck, Graveyard and
// banishment, and Sosei Ryu-Ge Mistva in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (93509766, DECK),
    (20904475, DECK),
    (56499179, DECK),
    (93509766, GRAVE),
    (20904475, GRAVE),
    (56499179, REMOVED),
    (20904475, REMOVED),
    (MISTVA, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_REMOVE + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_code(duel, MISTVA),
            a.tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DINOSAUR | RACE_SEASERPENT | RACE_WYRM)
        && (c.is_able_to_hand(duel) || c.is_able_to_remove(duel) || c.is_able_to_grave(duel))
}

fn thcheck(duel: &Duel, sg: CardSet) -> bool {
    aux::check_different_property(duel, sg, &|c, duel| vec![c.get_race(duel)])
        && sg.filter_count(duel, CardId::is_able_to_hand, None) >= 1
        && sg.filter_count(duel, CardId::is_able_to_remove, None) >= 1
        && sg.filter_count(duel, CardId::is_able_to_grave, None) >= 1
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| {
            (thcheck(duel, sg), false)
        }));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let sg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if sg.len() >= 3 {
        let g = aux::select_unselect_group(
            duel,
            sg,
            3,
            3,
            &|duel, sg, _, _| (thcheck(duel, sg), false),
            tp,
            aux::stringid(ID, 2),
        )?;
        if g.len() == 3 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let hg = g.filter_select(duel, tp, CardId::is_able_to_hand, 1, 1, None)?;
            if !hg.is_empty() && duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)? > 0 {
                duel.confirm_cards(1 - tp, hg)?;
                let g = g.difference(hg);
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
                let rg = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?;
                if !rg.is_empty() && duel.remove(rg, POS_FACEUP, REASON_EFFECT)? > 0 {
                    duel.send_to_grave(g.difference(rg), REASON_EFFECT)?;
                }
            }
        }
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !c.is_race(duel, RACE_DRAGON | RACE_DINOSAUR | RACE_SEASERPENT | RACE_WYRM)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE])
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcheck(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let res = aux::check_different_property(duel, sg, &|c, duel| vec![c.get_location(duel)])
        && aux::check_different_property(duel, sg, &|c, duel| vec![c.get_race(duel)]);
    (res, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, None);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| spcheck(duel, sg)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 3 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let location = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    let g = duel.get_matching_group(aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)), tp, location, 0, None);
    if g.len() < 3 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &|duel, sg, _, _| spcheck(duel, sg), tp, HINTMSG_SPSUMMON)?;
    if sg.len() == 3 {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
