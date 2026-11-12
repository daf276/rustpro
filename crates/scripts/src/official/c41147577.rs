// CXyz Simon the Great Moral Leader
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 41147577;
const NORITO: u32 = 14152862;
pub const LISTED_NAMES: &[u32] = &[NORITO];

// Xyz Summoned from the Extra Deck with three Level 7 Normal Monsters. The effect needs Norito the Moral Leader as material, an Xyz
// Monster, which no Xyz Summon can use.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6368038, MZONE),
    (6740720, MZONE),
    (24311372, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0)))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, poscon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel) && te.get_owner(duel) != e.get_owner(duel)
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_code(duel, NORITO), 1, None)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_change_position(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
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
    }
    Ok(())
}
