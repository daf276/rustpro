// Runick Freezing Curses
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_BECOME_TARGET;

const ID: u32 = 30430448;

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = CATEGORY_DISABLE;
    const RMCOUNT: usize = 3;
    const UNIQUEPROP: Option<u32> = Some(EFFECT_FLAG_CARD_TARGET);

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        negtg(duel, a, chk, chkc)
    }

    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool> {
        negop(duel, a)
    }
}

// Hugin the Runick Wings in the Extra Deck to Special Summon, and the opponent's effect monster to negate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (55990317, EXTRA), (13597785, MZONE | OPPONENT)];

pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(55990317), Check::Message(MSG_BECOME_TARGET)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Mysterune::create_qp_effect::<Quickplay>(duel, c);
    c.register_effect(duel, e1);
}

fn negfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EFFECT) && c.is_negatable_monster(duel)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && negfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(negfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, negfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<bool> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(false) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && !tc.is_disabled(duel) && !tc.is_immune_to_effect(duel, e)) {
        return Ok(false);
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    e2.set_value(duel, RESET_TURN_SET as i32);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e2);
    Ok(true)
}
