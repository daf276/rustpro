// Clown Crew Finale
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 84031736;

pub const LISTED_SERIES: &[u16] = &[SET_CLOWN_CREW];
// Set and in the Graveyard, with Clown Crew Biancaviso in the hand to Tribute Summon by Tributing a Clown Crew Flair on the
// field, one in the Deck to search with a different Level, and a Tribute Summoned monster to enable the first effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (82159583, HAND),
    (82159583, HAND),
    (82159583, DECK),
    (42759961, MZONE),
    (42759961, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_tribute_summoned, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::self_banish, thcost]));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -1);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    let c = e.get_handler(duel);
    let max_target_count = duel.get_target_count(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    let discard_count = duel.discard_hand(tp, aux::TRUE, 1, max_target_count as u8, REASON_COST | REASON_DISCARD)?;
    e.set_label(duel, i64::from(discard_count));
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        let cost_chk = e.get_label(duel) == -1;
        e.set_label(duel, 0);
        return Ok(cost_chk && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let target_count = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tg = duel.select_target(
        tp,
        CardId::is_faceup,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        target_count,
        target_count,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, tg.map_or(0, |tg| tg.len() as u8), tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn rating(c: CardId, duel: &Duel) -> u32 {
    if c.is_link_monster(duel) {
        c.get_link(duel)
    } else if c.is_xyz_monster(duel) {
        c.get_original_rank(duel)
    } else {
        c.get_original_level(duel)
    }
}

fn thcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(
        duel,
        TYPE_RITUAL | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_PENDULUM | TYPE_LINK,
    ) && {
        let lv = rating(c, duel);
        duel.is_existing_matching_card(|sc, duel| thfilter(sc, duel, lv), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)
    }
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_monster(duel) && c.is_able_to_hand(duel) && rating(c, duel) != lv
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -1);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| thcostfilter(c, duel, tp), (1, 1), true, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| thcostfilter(c, duel, tp), (1, 1), true, None, None)?;
    let Some(sc) = g.get_first() else { return Ok(false) };
    duel.release(sc, REASON_COST)?;
    e.set_label(duel, i64::from(rating(sc, duel)));
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let cost_chk = e.get_label(duel) == -1;
        e.set_label(duel, 0);
        return Ok(cost_chk);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let lv = e.get_label(duel) as u32;
    let filter = aux::necro_valley_filter(move |c, duel| thfilter(c, duel, lv));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        if sc.is_location(duel, LOCATION_GRAVE) {
            duel.hint_selection(sc);
        }
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        if sc.is_previous_location(duel, LOCATION_DECK) {
            duel.confirm_cards(1 - tp, sc)?;
        }
    }
    Ok(())
}
