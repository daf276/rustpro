// Runick Destruction
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, OPPONENT, SZONE};
use engine::constants::MSG_BECOME_TARGET;

const ID: u32 = 94445733;

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = CATEGORY_DESTROY;
    const RMCOUNT: usize = 4;
    const UNIQUEPROP: Option<u32> = Some(EFFECT_FLAG_CARD_TARGET);

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        destg(duel, a, chk, chkc)
    }

    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool> {
        desop(duel, a)
    }
}

// Hugin the Runick Wings in the Extra Deck to Special Summon, and the opponent's Spell to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (55990317, EXTRA), (44394295, SZONE | OPPONENT)];

pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(55990317), Check::Message(MSG_BECOME_TARGET)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Mysterune::create_qp_effect::<Quickplay>(duel, c);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, 1 - tp) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<bool> {
    let Some(tc) = duel.get_first_target() else { return Ok(false) };
    Ok(tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? > 0)
}
