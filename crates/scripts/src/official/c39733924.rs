// Full-Armored Xyz
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

use super::c73860462::xyz_summonable;

const ID: u32 = 39733924;

// Tornado Dragon in the Extra Deck for two Level 4 Normal Monsters, and Xyz Monsters in the field and the GY to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (6983839, EXTRA),
    (7359741, MZONE),
    (10992251, MZONE),
    (6983839, MZONE),
    (14152862, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(faceup_xyz, 0, LOCATION_MZONE, LOCATION_MZONE, 1, None)
    });
    e1.set_target(duel, xyztg);
    e1.set_operation(duel, xyzop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, _| !(duel.is_phase(PHASE_DAMAGE) && duel.is_damage_calculated()));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn faceup_xyz(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(xyz_summonable, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, xyz_summonable, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    {
        duel.xyz_summon(tp, sc, None, None)?;
    }
    Ok(())
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    faceup_xyz(c, duel) && !c.is_forbidden(duel)
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    faceup_xyz(c, duel) && duel.is_existing_matching_card(eqfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into()))
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let filter = aux::necro_valley_filter(eqfilter);
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, Some(tc.into()))?;
        if let Some(eq) = g.get_first()
            && duel.equip(tp, eq, tc)?
        {
            let e1 = duel.create_effect(eq);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_EQUIP_LIMIT);
            e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
            e1.set_label_object_group(duel, Some(tc.into()));
            e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            eq.register_effect(duel, e1);
            let atk = eq.get_text_attack(duel).max(0);
            let e2 = duel.create_effect(eq);
            e2.set_type(duel, EFFECT_TYPE_EQUIP);
            e2.set_code(duel, EFFECT_UPDATE_ATTACK);
            e2.set_value(duel, atk);
            e2.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            eq.register_effect(duel, e2);
            let e3 = duel.create_effect(eq);
            e3.set_type(duel, EFFECT_TYPE_EQUIP);
            e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
            e3.set_code(duel, EFFECT_DESTROY_SUBSTITUTE);
            e3.set_value(duel, Value::Reason(|_, _, _, r, _| r & (REASON_BATTLE | REASON_EFFECT) > 0));
            e3.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            eq.register_effect(duel, e3);
        }
    }
    Ok(())
}
