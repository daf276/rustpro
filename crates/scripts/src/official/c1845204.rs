// Instant Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 1845204;

// Fusionist and Invoked Caliga, Level 5 or lower Fusion Monsters that can be summoned without materials, in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (1641882, EXTRA), (13529466, EXTRA)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(1641882)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_FUSION)
        && c.get_level(duel) <= 5
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
        && c.check_fusion_material(duel, None, None, u64::from(PLAYER_NONE))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

struct Destroy;

impl aux::DelayedOperation for Destroy {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.destroy(ag, REASON_EFFECT)?;
        Ok(())
    }
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else {
        return Ok(());
    };
    tc.set_material(duel, None);
    if duel.special_summon(tc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? != 0 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3206);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        tc.complete_procedure(duel);
        aux::delayed_operation_reset::<Destroy>(duel, tc.into(), PHASE_END, 0, ID, e, tp);
    }
    Ok(())
}
