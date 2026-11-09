// Dogmatika Punishment
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 82956214;

// Set, with Extra Deck monsters of high ATK to send and a monster of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (22850702, EXTRA), (50954680, EXTRA), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && duel.is_existing_matching_card(|sc, duel| tgfilter(sc, duel, c.get_attack(duel)), tp, LOCATION_EXTRA, 0, 1, None)
}

fn tgfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_attack_above(duel, atk) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && desfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| desfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let atk = tc.get_attack(duel);
        let g = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, atk), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.send_to_grave(sc, REASON_EFFECT)? > 0
            && sc.is_location(duel, LOCATION_GRAVE)
        {
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let ct = if duel.is_turn_player(tp) { 2 } else { 1 };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, ct);
    duel.register_effect(e1, tp);
    Ok(())
}
