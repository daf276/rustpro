// Assault Mode Activate
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, SZONE};

const ID: u32 = 80280737;
const CRIMSON_BLADER: u32 = 80321197;
const CRIMSON_BLADER_ASSAULT_MODE: u32 = 50604072;

// Set with Crimson Blader on the field to Tribute and Crimson Blader/Assault Mode in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (CRIMSON_BLADER, MZONE),
    (CRIMSON_BLADER, MZONE),
    (CRIMSON_BLADER_ASSAULT_MODE, DECK),
    (CRIMSON_BLADER_ASSAULT_MODE, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(CRIMSON_BLADER_ASSAULT_MODE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, 1);
        Ok(true)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_synchro_monster(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_code(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_set_card(duel, &[SET_ASSAULT_MODE])
        && super::assault_mode(c.get_original_code(duel)) == Some(code)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| costfilter(c, duel, e, tp);
    if chk == 0 {
        if e.get_label(duel) != 1 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let sc = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?.get_first();
    if let Some(sc) = sc {
        duel.set_target_param(sc.get_code(duel));
        duel.release(sc, REASON_COST)?;
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, true, false, POS_FACEUP_ATTACK)? > 0
    {
        sc.complete_procedure(duel);
    }
    Ok(())
}
