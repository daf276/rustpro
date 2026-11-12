// Lady of the Lake
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 10736540;

// In the hand to Normal Summon and in the Graveyard to return, with Noble Knight Artorigus (a Normal Monster) in the
// Graveyard and Infernoble Knight Captain Roland (Level 5) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, GRAVE), (92125819, GRAVE), (40251688, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0a = duel.create_effect(c);
    e0a.set_type(duel, EFFECT_TYPE_SINGLE);
    e0a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0a.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e0a.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_race_except(duel, RACE_WARRIOR))));
    c.register_effect(duel, e0a);
    let e0b = e0a.clone_effect(duel);
    e0b.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e0b.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_reason(duel, REASON_SYNCHRO) && c.is_reason(duel, REASON_MATERIAL)
    });
    e0b.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e0b);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, gysptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NOBLE_KNIGHT]) && c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && gyspfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[5]) && c.is_set_card(duel, &[SET_NOBLE_KNIGHT]) && c.is_faceup(duel)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, 1, tp, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_level_above(duel, 2) {
        let c = e.get_handler(duel);
        let lv = tc.get_level(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        if tc.is_level(duel, &[lv - 1]) && c.is_relate_to_effect(duel, e) {
            duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
