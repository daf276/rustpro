// Rokket Synchron
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 48355999;

// In hand, with Power Vice Dragon (Level 5 DARK Dragon) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (19434243, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, sumtg);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 5)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_race(duel, RACE_DRAGON)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        let fid = c.get_field_id(duel);
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, i64::from(fid), 0);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e3.set_code(duel, EVENT_PHASE + PHASE_END);
        e3.set_count_limit(duel, 1, 0);
        e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e3.set_label(duel, i64::from(fid));
        e3.set_label_object_group(duel, Some(tc.into()));
        e3.set_condition_mut(duel, descon);
        e3.set_operation(duel, desop);
        duel.register_effect(e3, tp);
    }
    duel.special_summon_complete()?;
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_target_range(duel, 1, 0);
    e4.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_attribute(duel, ATTRIBUTE_DARK)
    });
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e4, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_attribute(duel, ATTRIBUTE_DARK));
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let e = a.e;
    let tc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.and_then(|tc| tc.get_flag_effect_label(duel, ID)) == Some(e.get_label(duel)) {
        true
    } else {
        e.reset(duel);
        false
    }
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
