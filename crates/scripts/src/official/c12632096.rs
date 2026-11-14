// Neo Galaxy-Eyes Cipher Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 12632096;

pub const LISTED_SERIES: &[u16] = &[SET_CIPHER];
pub const LISTED_NAMES: &[u32] = &[ID];

// Xyz Summoned from the Extra Deck with three Level 9 monsters (Dragonmaid Cehrmba, Lightstorm Dragon, Hot Red Dragon Archfiend
// Abyss). The effect needs a "Cipher" card as material, and the only ported ones are Xyz Monsters, which no Xyz Summon can use.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (12163590, MZONE),
    (10515412, MZONE),
    (9753964, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0)))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_CIPHER]), 1, None)
    });
    e1.set_cost(duel, cost);
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
}

fn control_filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_change_controler(duel)
}

fn ctcostmax(duel: &Duel, tp: u8) -> u16 {
    let ct = duel.get_matching_group_count(control_filter, tp, 0, LOCATION_MZONE, None) as i32;
    let ft = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_CONTROL, 0xff);
    ct.min(ft).clamp(0, 3) as u16
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let max = ctcostmax(duel, a.tp);
    Cost::detach_from_self_with(duel, a, chk, 1, max, |duel, e, og| e.set_label(duel, og.len() as i64))
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.e.get_label(duel) as u8;
    duel.set_operation_info(0, CATEGORY_CONTROL, None, count, a.tp, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let fid = if c.is_relate_to_effect(duel, e) {
        i64::from(c.get_field_id(duel))
    } else {
        -1
    };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e1.set_label(duel, fid);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let ct = e.get_label(duel).min(i64::from(duel.get_location_count_full(
        tp,
        LOCATION_MZONE,
        tp,
        LOCATION_REASON_CONTROL,
        0xff,
    ))) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_matching_card(tp, control_filter, tp, 0, LOCATION_MZONE, ct, ct, None)?;
    if g.len() == usize::from(ct) {
        // The core returns the control result as an integer, which is truthy in Lua even when 0.
        let reason_player = duel.get_reason_player();
        duel.get_control_full(g, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
        let og = duel.get_operated_group();
        for tc in og {
            tc.negate_effects(duel, c, RESET_CONTROL, false);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_value(duel, 4500);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CONTROL);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_CHANGE_CODE);
            e2.set_value(duel, ID as i32);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
