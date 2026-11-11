// Morphtronic Scopen
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 10591919;

// On the field with Level 4 Morphtronic Videon and Radion in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (84592800, HAND), (55119278, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, cona);
    e1.set_target(duel, tga);
    e1.set_operation(duel, opa);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CHANGE_LEVEL);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    e2.set_value(duel, 4);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.get_level(duel) == 4 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn cona(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.is_disabled(duel) && c.is_attack_pos(duel)
}

fn tga(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn opa(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    let c = e.get_handler(duel);
    let fid = c.get_field_id(duel);
    tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_label(duel, fid.into());
    e1.set_label_object_group(duel, Some(tc.into()));
    e1.set_condition_mut(duel, descon);
    e1.set_operation(duel, desop);
    duel.register_effect(e1, tp);
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    if tc.is_some_and(|tc| tc.get_flag_effect_label(duel, ID) != Some(a.e.get_label(duel))) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
