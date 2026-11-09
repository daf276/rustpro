// Double Snare
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 3682106;

// On the field with Blue-Eyes Shining Dragon and Dragunity Lord - Georgius, which carry the Double Snare marker effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (53347303, MZONE), (70522875, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_has_effect(duel, ID).is_some()
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel));
    }
    let exception = Some(a.e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, exception));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, exception)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
