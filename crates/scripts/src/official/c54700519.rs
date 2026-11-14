// Ursarctic Megatanus
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 54700519;

pub const LISTED_SERIES: &[u16] = &[SET_URSARCTIC];
pub const LISTED_NAMES: &[u32] = &[ID];
// Blue-Eyes White Dragon in the hand to Tribute, Ursarctic Megapolar and Megabilis on the field, and face-up monsters on the
// opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (89631139, HAND),
    (89631139, HAND),
    (28715905, MZONE),
    (81108658, MZONE),
    (89631140, MZONE | OPPONENT),
    (89631140, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ursarctic::add_sp_summon_quick_effect::<ID>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, poscon);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_URSARCTIC])
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
}

fn pfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(pfilter(chkc, duel) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(pfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, pfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
