// Therion "King" Regulus
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 10604644;

pub const LISTED_SERIES: &[u16] = &[SET_THERION];
// Copies in the hand and on the field (one is the discard cost), Overdrive (Machine) in the Graveyard to equip, and Elfnote Lucina
// on the opponent's side to negate. The equip effects work only while this card is equipped to a Therion monster, which no
// scenario does.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (2311603, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.rp == 1 - a.tp && duel.is_chain_disablable(a.ev as u8));
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_equip_target(duel)
            .is_some_and(|target| target.is_set_card(duel, &[SET_THERION]))
    });
    e3.set_value(duel, 700);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_target_card(duel, |duel, e, c| {
        Some(c) == e.get_handler(duel).get_equip_target(duel) && c.is_set_card(duel, &[SET_THERION])
    });
    e4.set_label_object(duel, Some(e2));
    c.register_effect(duel, e4);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && (c.is_set_card(duel, &[SET_THERION]) || c.is_race(duel, RACE_MACHINE))
}

fn eqtarget(c: CardId, duel: &Duel) -> bool {
    eqfilter(c, duel) && !c.is_forbidden(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && eqtarget(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(eqtarget, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, eqtarget, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e) && tc.is_monster(duel) && !tc.is_forbidden(duel))
    {
        c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, None)?;
    }
    Ok(())
}

fn discostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THERION])
        && c.is_original_type(duel, TYPE_MONSTER)
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(discostfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, discostfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    if chk == 0 {
        return Ok(!re.get_handler(duel).is_disabled(duel));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_effect(a.ev as u8);
    Ok(())
}
