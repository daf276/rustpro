// Junk Changer
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 1006081;

// Junk Synchron on the field to raise or lower the Level of.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (63977008, MZONE), (63977008, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && c.is_set_card(duel, &[SET_JUNK])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EFFECT);
    let op = if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.get_level(duel) == 1) {
        duel.select_option(tp, &[aux::stringid(ID, 1)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
    };
    a.e.set_label(duel, i64::from(op));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, if e.get_label(duel) == 0 { 1 } else { -1 });
        tc.register_effect(duel, e1);
    }
    Ok(())
}
