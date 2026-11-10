// Wedju Temple
use super::c38775407::treat_as_continuous_spell;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 63017368;

// In the hand to activate with Millennium Shield in the hand and Deck to place, and Sengenjin on the field for Dark Hole to destroy.
// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (32012841, HAND),
    (32012841, DECK),
    (32012841, DECK),
    (38775407, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target(duel, reptg);
    e3.set_operation(duel, repop);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)) && c.get_flag_effect(duel, ID) > 0)),
    );
    c.register_effect(duel, e3);
}

fn hplfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn dplfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MILLENNIUM]) && hplfilter(c, duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 1
        && duel.is_existing_matching_card(|c, duel| hplfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None)
        && duel.is_existing_matching_card(|c, duel| dplfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let hg = duel.select_matching_card(tp, |c, duel| hplfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(hc) = hg.get_first()
        && place(duel, hc, c, tp)?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let dg = duel.select_matching_card(tp, |c, duel| dplfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        let Some(dc) = dg.get_first() else { return Ok(()) };
        duel.break_effect()?;
        place(duel, dc, c, tp)?;
    }
    Ok(())
}

fn place(duel: &mut Duel, c: CardId, rc: CardId, tp: u8) -> Script<bool> {
    if !duel.move_to_field(c, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
        return Ok(false);
    }
    treat_as_continuous_spell(duel, rc, c);
    Ok(true)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MILLENNIUM])
        && c.is_faceup(duel)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT | REASON_BATTLE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut g = a.eg.unwrap_or_default().filter(duel, |c, duel| repfilter(c, duel, tp), None);
    let mut ft = duel.get_location_count(tp, LOCATION_SZONE);
    if chk == 0 {
        return Ok(ft > 0 && !g.is_empty() && !e.get_handler(duel).is_status(duel, STATUS_DESTROY_CONFIRMED));
    }
    if !duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        return Ok(false);
    }
    if g.len() > 1 {
        ft = ft.min(g.len() as i32);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        g = g.select(duel, tp, 1, ft as u8, None)?;
    }
    for sc in g {
        sc.register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD), 0, 1);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    for tc in a.e.get_label_object_group(duel).unwrap_or_default() {
        if duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, tc.is_monster_card(duel))? {
            treat_as_continuous_spell(duel, c, tc);
        }
    }
    Ok(())
}
