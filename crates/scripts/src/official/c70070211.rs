// Joruri-P.U.N.K. Nashiwari Surprise
use crate::prelude::*;
use crate::testing::{Check, MZONE, SZONE};

const ID: u32 = 70070211;

pub const LISTED_SERIES: &[u16] = &[SET_PUNK];

// Set, with Ze Amin on the field to target face-up cards too.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (19535693, MZONE), (19535693, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, destg);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PUNK]) && c.is_faceup(duel)
}

fn tgfilter(c: CardId, duel: &Duel, punk: bool) -> bool {
    c.is_facedown(duel) || punk
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let punk = duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None);
    let tgfilter = |c, duel: &Duel| tgfilter(c, duel, punk);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && tgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tc = duel.select_target(tp, tgfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, tc, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}
