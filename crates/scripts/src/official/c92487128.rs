// Sosei Ryu-Ge Mistva
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT, PZONE};

const ID: u32 = 92487128;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
pub const LISTED_NAMES: &[u32] = &[ID];
// In a Pendulum Zone to search Ryu-Ge Realms and destroy itself, with Genro Ryu-Ge Hakva (Level 10) on the field to
// Tribute, and the opponent's Sangans to be destroyed in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (56499179, MZONE),
    (56499179, MZONE),
    (55154344, DECK),
    (28669235, DECK),
    (82661630, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure(duel, c);
    c.add_must_be_special_summoned(duel);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && !c.is_type(duel, TYPE_PENDULUM) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        let c = a.e.get_handler(duel);
        if !c.is_relate_to_effect(duel, a.e) {
            return Ok(());
        }
        duel.break_effect()?;
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.contains(a.e.get_handler(duel))
        && eg.is_exists(
            duel,
            |c, duel| c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT),
            1,
            None,
        )
}

fn tributefilter(c: CardId, duel: &Duel, tp: u8, hc: CardId) -> bool {
    c.is_level(duel, &[10])
        && c.is_set_card(duel, &[SET_RYU_GE])
        && c.is_releasable_by_effect(duel)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), hc) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|tc, duel| tributefilter(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, None)
                && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_RITUAL, tp, true, true),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_continuous_spell(duel) && !c.is_forbidden(duel)
}

fn tfcheck(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let res =
        duel.get_location_count(tp, LOCATION_SZONE) > 0 || sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_STZONE), 1, None);
    (res, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let rg = duel.select_matching_card(tp, |tc, duel| tributefilter(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if rg.is_empty() || duel.release(rg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, SUMMON_TYPE_RITUAL, tp, tp, true, true, POS_FACEUP)? == 0 {
        return Ok(());
    }
    c.complete_procedure(duel);
    let fg = duel.get_field_group(tp, LOCATION_ONFIELD, 0);
    let dg = duel.get_matching_group(plfilter, tp, LOCATION_DECK, 0, None);
    let check = |duel: &Duel, sg, _, _| tfcheck(duel, sg, tp);
    if !fg.is_empty()
        && !dg.is_empty()
        && aux::select_unselect_group_check(duel, fg, 1, 2, &check)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        let desg = aux::select_unselect_group_full(duel, fg, 1, 2, &check, tp, HINTMSG_DESTROY, Some(&check), false)?;
        duel.hint_selection(desg);
        duel.break_effect()?;
        let ct = duel.destroy(desg, REASON_EFFECT)?;
        let ft = duel.get_location_count(tp, LOCATION_SZONE);
        let ct = ct.min(ft);
        if ct == 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let placeg = dg.select(duel, tp, 1, ct as u8, None)?;
        for tc in placeg {
            duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
        }
    }
    Ok(())
}
