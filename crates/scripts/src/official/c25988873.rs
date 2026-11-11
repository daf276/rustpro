// Dragunity Partisan
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 25988873;

// Copies in hand to Normal Summon, and Dragunity Tribus (Winged Beast) in hand to Special Summon and equip to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE), (81962318, HAND), (81962318, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_ADD_TYPE);
    e2.set_value(duel, TYPE_TUNER as i32);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGUNITY]) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    special_summon_and_equip(duel, a, |c, duel| filter(c, duel, e, tp))
}

// The Special Summon and equip of Partisan, Aklys and Pilum, which differ in the filter.
pub fn special_summon_and_equip(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    let c = e.get_handler(duel);
    if c.is_facedown(duel)
        || !c.is_relate_to_effect(duel, e)
        || c.is_controler(duel, 1 - tp)
        || duel.get_location_count(tp, LOCATION_SZONE) <= 0
    {
        return Ok(());
    }
    duel.break_effect()?;
    if !duel.equip_full(tp, c, tc, false, false)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e1.set_label_object_group(duel, Some(tc.into()));
    c.register_effect(duel, e1);
    Ok(())
}
