// Joruri-P.U.N.K. Dangerous Gabu
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 43685562;

pub const LISTED_SERIES: &[u16] = &[SET_PUNK];
// Set, with Gagaku-P.U.N.K. Wa Gon on the field to recover with and the opponent's Sangan to negate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (50642380, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn negatable(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_effect_monster(duel)
}

fn punk_faceup(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_PUNK])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && negatable(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(negatable, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let tc = duel
        .select_target(tp, negatable, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first());
    // The Lua passes an undefined global `g` as the disabled card.
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, tp, 0);
    let atk = tc.map_or(0, |tc| tc.get_base_attack(duel));
    if duel.is_existing_matching_card(punk_faceup, tp, LOCATION_MZONE, 0, 1, None) {
        duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, atk);
    }
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, atk);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_can_be_disabled_by_effect(duel, e, true) {
        tc.negate_effects(duel, e.get_handler(duel), RESETS_STANDARD_PHASE_END, false);
        if !duel.is_existing_matching_card(punk_faceup, tp, LOCATION_MZONE, 0, 1, None) {
            return Ok(());
        }
        duel.adjust_instantly(Some(tc));
        let atk = tc.get_base_attack(duel);
        if tc.is_disabled(duel) && atk > 0 {
            duel.break_effect()?;
            duel.recover(tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
