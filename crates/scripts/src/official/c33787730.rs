// Ryzeal Plasma Hole
use super::c60394026::{attach, attachfilter};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 33787730;

pub const LISTED_SERIES: &[u16] = &[SET_RYZEAL];
// In hand and the Graveyard, Ryzeal Detonator and Ryzeal monsters on the field to count and to Xyz Summon it from the Extra
// Deck, Ryzeal monsters in the Graveyard to attach, and face-up opponent monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (34909328, MZONE),
    (34909328, EXTRA),
    (8633261, MZONE),
    (35844557, MZONE),
    (72238166, GRAVE),
    (34022970, GRAVE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, xyztg);
    e2.set_operation(duel, xyzop);
    c.register_effect(duel, e2);
}

fn desconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc.is_controler(duel, 1 - tp));
    }
    let ct = duel.get_matching_group_count(desconfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, ct as u8, Some(c.into()))?;
    let count = dg.map_or(0, |dg| dg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DESTROY, dg, count, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    let filter = aux::necro_valley_filter(move |c, duel| attachfilter(c, duel, tp));
    if !g.is_empty()
        && duel.destroy(g, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        let Some(mc) = duel
            .select_matching_card(tp, &filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(mc);
        attach(duel, e, tp, mc)?;
    }
    Ok(())
}

fn raizeolmatfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL])
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |xc, duel| xc.is_xyz_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| raizeolmatfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(tp, |c, duel| raizeolmatfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(
        tp,
        |xc, duel| xc.is_xyz_summonable(duel, Some(g), None),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(xyzc) = sg.get_first() {
        duel.xyz_summon(tp, xyzc, Some(g), None)?;
    }
    Ok(())
}
