// Xyz Remora
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 43138260;

// Tornado Dragon with Xyz materials to detach, and two Great Whites in the GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (6983839, MZONE), (13429800, GRAVE), (13429800, GRAVE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.check_remove_overlay_card(tp, 1, 0, 2, REASON_COST)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove_overlay_card(a.tp, 1, 0, 2, 2, REASON_COST)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FISH)
        && c.get_level(duel) == 4
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 2, 0, 0);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.is_empty()
        || duel.get_location_count(tp, LOCATION_MZONE) < g.len() as i32
        || (g.len() > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some())
    {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for tc in g {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        let reset = RESET_EVENT + RESETS_STANDARD;
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, reset);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE);
        e2.set_reset(duel, reset);
        tc.register_effect(duel, e2);
        let e3 = e2.clone_effect(duel);
        e3.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e3);
        let e4 = duel.create_effect(c);
        e4.set_description(duel, 3313);
        e4.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e4.set_type(duel, EFFECT_TYPE_SINGLE);
        e4.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
        e4.set_reset(duel, reset);
        tc.register_effect(duel, e4);
        let e5 = duel.create_effect(c);
        e5.set_type(duel, EFFECT_TYPE_SINGLE);
        e5.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
        e5.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_attribute(duel, ATTRIBUTE_WATER))));
        e5.set_reset(duel, reset);
        tc.register_effect(duel, e5);
    }
    duel.special_summon_complete()?;
    Ok(())
}
