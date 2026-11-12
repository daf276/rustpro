// Rider of the Storm Winds
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 14235211;

// In the hand and on the field, with Dragon Normal Monsters to equip to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (88819587, MZONE), (75356564, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_DESTROY_SUBSTITUTE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_NORMAL)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || (c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel)) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0
        || tc.get_controler(duel) != tp
        || tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    }
    duel.equip_full(tp, c, tc, true, false)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e1.set_label_object_group(duel, Some(tc.into()));
    c.register_effect(duel, e1);
    Ok(())
}
