// D/D/D Headhunt
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 91781484;

pub const LISTED_SERIES: &[u16] = &[SET_DDD];
// D/D/D Abyss King Gilgamesh, a Link Monster and so face-up, with Dark Blade on the opponent's field to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (9024198, MZONE),
    (9024198, MZONE),
    (11321183, MZONE | OPPONENT),
    (11321183, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DDD]));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_faceup(duel)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_controler_can_be_changed(duel));
    }
    let filter = aux::faceup_filter(CardId::is_controler_can_be_changed);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    duel.get_control_full(tc, tp, PHASE_END as u16, 2, 0xff, duel.get_reason_player())?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    tc.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK);
    tc.register_effect(duel, e3);
    if tc.is_special_summoned(duel) && tc.is_summon_location(duel, LOCATION_EXTRA) {
        let e4 = duel.create_effect(c);
        e4.set_description(duel, aux::stringid(ID, 1));
        e4.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e4.set_type(duel, EFFECT_TYPE_SINGLE);
        e4.set_code(duel, EFFECT_ADD_SETCODE);
        e4.set_value(duel, SET_DDD as i32);
        e4.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e4);
    }
    Ok(())
}
