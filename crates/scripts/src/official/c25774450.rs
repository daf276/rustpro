// Mystic Box
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 25774450;

// In the hand, with monsters on both sides: one of the opponent's to destroy, one of yours to give.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (756652, MZONE),
    (756652, MZONE),
    (15401633, MZONE | OPPONENT),
    (15401633, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| filter(c, duel, 1 - tp), tp, 0, LOCATION_MZONE, 1, None)
                && duel.is_existing_target(CardId::is_able_to_change_controler, tp, LOCATION_MZONE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_target(tp, |c, duel| filter(c, duel, 1 - tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g2 = duel.select_target(tp, CardId::is_able_to_change_controler, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g1, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_CONTROL, g2, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let dc = duel
        .get_operation_info(0, CATEGORY_DESTROY)
        .and_then(|info| info.op_cards)
        .and_then(|g| g.get_first());
    let cc = duel
        .get_operation_info(0, CATEGORY_CONTROL)
        .and_then(|info| info.op_cards)
        .and_then(|g| g.get_first());
    let (Some(dc), Some(cc)) = (dc, cc) else {
        unported!(duel, true);
        return Ok(());
    };
    if dc.is_relate_to_effect(duel, e) && duel.destroy(dc, REASON_EFFECT)? > 0 && cc.is_relate_to_effect(duel, e) {
        duel.break_effect()?;
        duel.get_control(cc, 1 - tp)?;
    }
    Ok(())
}
