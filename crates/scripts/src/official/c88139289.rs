// Psychic Blaster Mk-II
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, REMOVED};

const ID: u32 = 88139289;

// In the Extra Deck, Synchro Summoned with Stinger the Electric Expeller (Level 3 Psychic Tuner) and two Chosen by the World Chalice (Level 3 Psychic), with monsters in the Graveyard and banished.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (57649113, MZONE),
    (22916281, MZONE),
    (22916281, MZONE),
    (10071456, GRAVE),
    (10071456, GRAVE),
    (10071456, REMOVED),
    (10071456, REMOVED),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, rmvcost);
    e2.set_target(duel, rmvtg);
    e2.set_operation(duel, rmvop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, LOCATION_REMOVED, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_REMOVED,
        LOCATION_REMOVED,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn rmvcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_GRAVE)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel) && chkc.is_able_to_remove(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            aux::faceup_filter(CardId::is_able_to_remove),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(
        tp,
        aux::faceup_filter(CardId::is_able_to_remove),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        Some(c.into()),
    )?;
    let Some(tc) = g.and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tc.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 1, tp, tc.get_base_attack(duel));
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0 && tc.is_location(duel, LOCATION_REMOVED) {
        let atk = tc.get_base_attack(duel);
        if atk > 0 {
            duel.break_effect()?;
            duel.recover(a.tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
