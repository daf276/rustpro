// Maestroke the Symphony Djinn
use super::c11646785::setop as posop;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 25341652;

// In the Extra Deck, Xyz Summoned with two Level 4 monsters (Hyosube, Water Omotics), an opponent's Attack Position monster
// (Elfnote Lucina) to turn face-down, and Raigeki in the opponent's hand to destroy a Djinn and replace it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (2483611, MZONE),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    e2.set_operation(duel, repop);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_can_turn_set(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(posfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, posfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_set_card(duel, &[SET_DJINN])
        && !c.is_reason(duel, REASON_REPLACE)
        && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    if duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        let g = eg.filter(duel, |c, duel| repfilter(c, duel, tp), None);
        e.set_label_object_group(duel, Some(g));
        return Ok(true);
    }
    Ok(false)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    for tc in a.e.get_label_object_group(duel).unwrap_or_default().iter() {
        tc.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    }
    Ok(())
}
