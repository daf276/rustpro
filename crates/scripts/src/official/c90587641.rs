// Superheavy Samurai Commander Shanawo
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, PZONE};

const ID: u32 = 90587641;

pub const LISTED_SERIES: &[u16] = &[SET_SUPERHEAVY_SAMURAI];

// Masurawo, Minerva, Grand Partisan, Ballista and Hama reuse it.
pub struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// A copy in the Pendulum Zone for the opponent's Elfnote Lucina to attack into, one in the Extra Deck, Synchro Summoned from
// Drum (a Level 1 Tuner) and Dark Magician (Level 7), and Superheavy Samurai in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (87535691, MZONE),
    (36996508, MZONE),
    (87535691, GRAVE),
    (83334932, GRAVE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|at| at.is_controler(duel, 1 - a.tp))
    });
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        duel.is_battle_phase() && a.e.get_handler(duel).is_synchro_summoned(duel)
    });
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let at = duel.get_attacker();
    unported!(duel, at.is_none());
    let Some(at) = at else { return Ok(false) };
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    let atk = at.get_attack(duel);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, Some(at.into()), 1, tp, -atk);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_attacker();
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && !duel.is_existing_matching_card(CardId::is_spell_trap, tp, LOCATION_GRAVE, 0, 1, None)
    {
        unported!(duel, tc.is_none());
        let Some(tc) = tc else { return Ok(()) };
        if tc.is_relate_to_battle(duel) && tc.is_faceup(duel) && tc.is_controler(duel, 1 - tp) {
            duel.break_effect()?;
            duel.negate_related_chain(tc, RESET_TURN_SET);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_value(duel, 0);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_DISABLE);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e2);
            let e3 = e2.clone_effect(duel);
            e3.set_code(duel, EFFECT_DISABLE_EFFECT);
            e3.set_value(duel, RESET_TURN_SET as i32);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI])
        && c.is_original_type(duel, TYPE_MONSTER)
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_GRAVE | LOCATION_STZONE,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_STZONE, 0, 1, 1, None)?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let c = e.get_handler(duel);
        if !(c.is_relate_to_effect(duel, e) && c.is_type(duel, TYPE_PENDULUM)) {
            return Ok(());
        }
        if duel.check_pendulum_zones(tp) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
        }
    }
    Ok(())
}
