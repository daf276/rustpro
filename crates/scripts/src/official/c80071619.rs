// R.B. Shepherd's Crook
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 80071619;

pub const LISTED_SERIES: &[u16] = &[SET_RB];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_set_card_full(duel, &[SET_RB], Some(lc), sumtype, tp), 1, None)
    }
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_RB];

// In the Extra Deck, Link Summoned from R.B. VALCan Booster (a Link-2) and R.B. Lambda Cannon, with more Lambda Cannons in the Graveyard
// to shuffle and Special Summon during the opponent's Main Phase.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (6821579, MZONE),
    (81794107, MZONE),
    (81794107, GRAVE),
    (81794107, GRAVE),
    (81794107, GRAVE),
    (30655537, MZONE | OPPONENT),
    (79859067, DECK),
    (79859067, DECK),
    (79859067, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| {
            500 * duel.get_matching_group_count(
                aux::TRUE,
                c.get_controler(duel),
                LOCATION_MZONE,
                0,
                Some(e.get_handler(duel).into()),
            ) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| duel.is_main_phase_full(Some(1 - a.tp)));
    e3.set_target(duel, tdsptg);
    e3.set_operation(duel, tdspop);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e3);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RB]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(setfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}

fn tdspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 3)
        && c.is_set_card(duel, &[SET_RB])
        && c.is_can_be_effect_target(duel, e)
        && (c.is_able_to_deck(duel) || c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff))
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (sg.is_exists(duel, |c, duel| spcheck(c, duel, e, tp, sg), 1, None), false)
}

fn spcheck(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && sg.is_exists(duel, CardId::is_able_to_deck, 2, Some(c.into()))
}

fn tdsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tdspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && g.len() >= 3
            && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    let tg = aux::select_unselect_group(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, e, tp), tp, HINTMSG_TARGET)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 1, tp, 0);
    Ok(false)
}

fn tdrescon(duel: &Duel, sg: CardSet, mg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (mg.is_exists(duel, |c, duel| spcheck(c, duel, e, tp, sg), 1, Some(sg)), false)
}

fn tdspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.len() < 2 {
        return Ok(());
    }
    let in_deck = |tg: CardSet, duel: &Duel| tg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    if tg.len() == 2 && duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0 {
        let ct = in_deck(tg, duel);
        if ct > 0 {
            duel.sort_deckbottom(tp, tp, ct as u16)?;
        }
        return Ok(());
    }
    let dg = aux::select_unselect_group(duel, tg, 2, 2, &|duel, sg, mg, _| tdrescon(duel, sg, mg, e, tp), tp, HINTMSG_TODECK)?;
    if dg.len() != 2 {
        return Ok(());
    }
    duel.hint_selection(dg);
    if duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && dg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None) == 2
    {
        let ct = in_deck(dg, duel);
        if ct > 0 {
            duel.sort_deckbottom(tp, tp, ct as u16)?;
        }
        let sg = tg.difference(dg);
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}
