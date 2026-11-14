// Ursarctic Megapolar
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 28715905;

pub const LISTED_SERIES: &[u16] = &[SET_URSARCTIC];
pub const LISTED_NAMES: &[u32] = &[ID];
// Blue-Eyes White Dragon in the hand to Tribute, Ursarctic Megabilis and Megatanus on the field, and Mirror Force on the
// opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (89631139, HAND),
    (89631139, HAND),
    (81108658, MZONE),
    (54700519, MZONE),
    (44095762, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ursarctic::add_sp_summon_quick_effect::<ID>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, popcon);
    e2.set_target(duel, poptg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_URSARCTIC])
}

fn pfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_destructable(duel)
}

fn popcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
}

fn poptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(pfilter(chkc, duel) && chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(pfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, pfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}
