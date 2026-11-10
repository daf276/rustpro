// Tragoedia
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 98777036;

// In the hand and on the field, with Sangan (Level 3) in the hand to send and in the Graveyard to copy the Level of, and two
// of the opponent's to take control of or to attack for the battle damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (26202165, HAND),
    (26202165, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, |_, a| a.ep == a.tp);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| duel.get_field_group_count(c.get_controler(duel), LOCATION_HAND, 0) as i32 * 600),
    );
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_CONTROL);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, ctcost);
    e4.set_target(duel, cttg);
    e4.set_operation(duel, ctop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_LVCHANGE);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_target(duel, lvtg);
    e5.set_operation(duel, lvop);
    c.register_effect(duel, e5);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn ctfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_controler_can_be_changed(duel) && c.is_faceup(duel) && c.is_level(duel, &[lv])
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let lv = c.get_level(duel);
    c.is_monster(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_target(|c, duel| ctfilter(c, duel, lv), tp, 0, LOCATION_MZONE, 1, None)
}

fn ctcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sg = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(first) = sg.get_first() {
        let lv = first.get_level(duel);
        a.e.set_label(duel, lv.into());
    }
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lv = a.e.get_label(duel) as u32;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && ctfilter(chkc, duel, lv));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, |c, duel| ctfilter(c, duel, lv), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        duel.get_control(tc, a.tp)?;
    }
    Ok(())
}

fn lvlfilter(c: CardId, duel: &Duel, lvl: u32) -> bool {
    c.has_level(duel) && !c.is_level(duel, &[lvl])
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lvl = a.e.get_handler(duel).get_level(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && lvlfilter(chkc, duel, lvl));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| lvlfilter(c, duel, lvl), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| lvlfilter(c, duel, lvl), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e)) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
