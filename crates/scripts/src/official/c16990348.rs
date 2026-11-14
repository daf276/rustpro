// Gladiator Beast Charge
use crate::prelude::*;
use crate::testing::{MZONE, SZONE};

const ID: u32 = 16990348;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
// Set, with Gladiator Beast Hoplomus and Andal (different names) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (4253484, MZONE), (90582719, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn gladiator(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GLADIATOR])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(gladiator, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(gladiator, tp, LOCATION_MZONE, 0, None);
    let ct = g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let sg = duel.select_target(
        tp,
        CardId::is_faceup,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        ct as u8,
        Some(c.into()),
    )?;
    let count = sg.map_or(0, |sg| sg.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, sg, count, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
