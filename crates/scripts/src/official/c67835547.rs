// Runick Slumber
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_BECOME_TARGET;

const ID: u32 = 67835547;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = 0;
    const RMCOUNT: usize = 3;
    const UNIQUEPROP: Option<u32> = Some(EFFECT_FLAG_CARD_TARGET);

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        natktg(duel, a, chk, chkc)
    }

    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool> {
        natkop(duel, a)
    }
}

// Hugin the Runick Wings in the Extra Deck to Special Summon, and the opponent's monster to affect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (55990317, EXTRA), (13597785, MZONE | OPPONENT)];

pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(55990317), Check::Message(MSG_BECOME_TARGET)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Mysterune::create_qp_effect::<Quickplay>(duel, c);
    c.register_effect(duel, e1);
}

fn natktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn natkop(duel: &mut Duel, a: &Args) -> Script<bool> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(false) };
    if !(tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e)) {
        return Ok(false);
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & (REASON_BATTLE + REASON_EFFECT) != 0));
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e2);
    Ok(true)
}
