// Penguin Squire
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 14761450;

pub const LISTED_SERIES: &[u16] = &[SET_PENGUIN];
// Three copies in the hand and Normal Monsters to Set, with a Set monster on the field for the position change.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (48531733, HAND),
    (48531733, HAND),
    (5628232, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_attribute(duel, ATTRIBUTE_WATER))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHANGE_POS);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon1);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_MSET);
    e3.set_condition(duel, spcon2);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_condition(duel, spcon3);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_target(duel, postg);
    e5.set_operation(duel, posop);
    c.register_effect(duel, e5);
}

fn filter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP) && c.is_facedown(duel) && c.is_controler(duel, tp)
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| filter1(c, duel, a.tp), 1, None)
}

fn filter2(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_facedown(duel) && c.is_controler(duel, tp)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| filter2(c, duel, a.tp), 1, None)
}

fn filter3(c: CardId, duel: &Duel, tp: u8) -> bool {
    filter2(c, duel, tp) && c.is_location(duel, LOCATION_MZONE)
}

fn spcon3(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| filter3(c, duel, a.tp), 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
        let lv = duel.announce_number_range(tp, 1, 2)?;
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, -(lv as i32));
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_defense_pos(duel) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(posfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEDOWNDEFENSE);
    duel.select_target(tp, posfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.change_position(tc, POS_FACEUP_DEFENSE)? > 0
        && tc.is_faceup(duel)
        && !tc.is_set_card(duel, &[SET_PENGUIN])
    {
        tc.negate_effects(duel, c, RESET_EVENT | RESETS_STANDARD, false);
    }
    Ok(())
}
