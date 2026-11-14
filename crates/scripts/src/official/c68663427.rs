// Reincarnation Unveiling Mail
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 68663427;

pub const LISTED_SERIES: &[u16] = &[SET_ARMORED_XYZ];
// Tornado Dragon to equip to, and Xyz Armor Torpedo with two Level 3 Normal Monsters (Swordsman of Landstar) for the Xyz Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (6983839, MZONE),
    (94151981, EXTRA),
    (3573512, MZONE),
    (3573512, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::Message(MSG_EQUIP)];

struct Procedure;

impl EquipProcedure for Procedure {}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = Equip::add_procedure::<Procedure>(duel, c);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e1b.set_range(duel, LOCATION_SZONE);
    e1b.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1b.set_target_card(duel, equipped_xyz);
    e1b.set_label_object(duel, Some(e1));
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_target(duel, xyztg);
    e2.set_operation(duel, xyzop);
    let e2b = e1b.clone_effect(duel);
    e2b.set_label_object(duel, Some(e2));
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_EQUIP);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_END);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_turn_id(duel) == duel.get_turn_count());
    e3.set_target(duel, eqptg);
    e3.set_operation(duel, eqpop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_ADD_TYPE);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_target_card(duel, equipped_xyz);
    e4.set_value(duel, TYPE_EFFECT as i32);
    c.register_effect(duel, e4);
}

fn equipped_xyz(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_equip_target(duel) == Some(c) && c.is_type(duel, TYPE_XYZ)
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_xyz_summonable(duel, None, None)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let eqpg = a.e.get_handler(duel).get_equip_group(duel);
    if chk == 0 {
        return Ok(!eqpg.is_empty()
            && eqpg.is_exists(duel, CardId::is_able_to_hand, 1, None)
            && duel.is_existing_matching_card(xyzfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(eqpg), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = c.get_equip_group(duel);
    if !c.is_relate_to_effect(duel, e) || g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let Some(rtc) = g.filter_select(duel, tp, CardId::is_able_to_hand, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    duel.hint_selection(rtc);
    if duel.send_to_hand(rtc, PLAYER_NONE, REASON_EFFECT)? > 0 && rtc.is_location(duel, LOCATION_HAND) {
        let sg = duel.get_matching_group(xyzfilter, tp, LOCATION_EXTRA, 0, None);
        if !sg.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sc = sg.select(duel, tp, 1, 1, None)?.get_first();
            duel.break_effect()?;
            if let Some(sc) = sc {
                duel.xyz_summon(tp, sc, None, None)?;
            }
        }
    }
    Ok(())
}

fn faceup_xyz(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn eqptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_matching_card(faceup_xyz, tp, LOCATION_MZONE, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn eqpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    if let Some(sc) = duel
        .select_matching_card(tp, faceup_xyz, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    {
        duel.hint_selection(sc);
        duel.equip(tp, c, sc)?;
    }
    Ok(())
}
