// Vanquish Soul Dust Devil
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 91951471;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// Vanquish Soul monsters on the field to change, and face-up opponent's monsters to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (29302858, MZONE),
    (29280200, MZONE),
    (93156774, MZONE),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_can_change_position(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, tgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, 1 - tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let positions = [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK];
    if tc.is_relate_to_effect(duel, a.e) && duel.change_position_full(tc, positions, false)? > 0 {
        let ct = duel
            .get_matching_group(
                aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_VANQUISH_SOUL])),
                tp,
                LOCATION_MZONE,
                0,
                None,
            )
            .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
        let g = duel.get_matching_group(aux::faceup_filter(CardId::is_can_turn_set), tp, 0, LOCATION_MZONE, None);
        if ct > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let fg = g.select(duel, tp, 1, ct as u8, None)?;
            duel.hint_selection_full(fg, true);
            duel.break_effect()?;
            duel.change_position(fg, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}
