// Argostars - Giantslaying
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 30964246;

pub const LISTED_SERIES: &[u16] = &[SET_ARGOSTARS];
// Argostars monsters to add, a Lightning Tydeu to activate, and a copy in the Graveyard to return cards with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (91438674, DECK),
    (91438674, DECK),
    (91438674, HAND),
    (65889305, SZONE),
    (65889305, SZONE),
    (91438674, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, retthtg);
    e2.set_operation(duel, retthop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARGOSTARS]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    Ok(false)
}

fn trapfilter(c: CardId, duel: &Duel) -> bool {
    c.is_continuous_trap(duel)
        && (c.is_location(duel, LOCATION_MZONE) || (c.is_trap_monster(duel) && c.is_location(duel, LOCATION_STZONE)))
        && c.is_faceup(duel)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.is_summonable(duel, true, None)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(tc) = duel
        .select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0 && tc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, tc)?;
        duel.shuffle_hand(tp)?;
        if duel.is_existing_matching_card(trapfilter, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
            if let Some(sc) = duel
                .select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
                .get_first()
            {
                duel.break_effect()?;
                duel.summon(tp, sc, true, None)?;
            }
        }
    }
    Ok(())
}

fn argstrapfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ARGOSTARS]) && c.is_continuous_trap(duel) && c.is_controler(duel, tp) && c.is_faceup(duel)
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| (sg.is_exists(duel, |c, duel| argstrapfilter(c, duel, tp), 1, None), false)
}

fn retthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let rg = duel.get_matching_group(
        |c, duel| c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    if chk == 0 {
        return Ok(rg.len() >= 2 && aux::select_unselect_group_check(duel, rg, 2, 2, &rescon(tp)));
    }
    let g = aux::select_unselect_group(duel, rg, 2, 2, &rescon(tp), tp, HINTMSG_RTOHAND)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn retthop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
