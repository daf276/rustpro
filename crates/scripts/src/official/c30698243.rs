// Red Hypernova Dragon
use super::c97489701::atkval;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 30698243;
const RED_NOVA_DRAGON: u32 = 97489701;

pub const LISTED_NAMES: &[u32] = &[RED_NOVA_DRAGON];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN1: usize = 4;
    const MAX1: usize = 4;
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with four Level 1 Tuners (Water Spirit) and Red Dragon Archfiend on the field, then
// banishing itself to Special Summon Red Nova Dragon from the Extra Deck, with Tuners in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (RED_NOVA_DRAGON, EXTRA),
    (ID, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (487395, MZONE),
    (487395, MZONE),
    (487395, MZONE),
    (487395, MZONE),
    (17266660, GRAVE),
    (17266660, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_MULTIPLE_TUNERS);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, Value::Card(|duel, _, c| atkval(duel, c)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_REMOVE | CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_QUICK_O);
    e5.set_code(duel, EVENT_FREE_CHAIN);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e5.set_target(duel, rmtg);
    e5.set_operation(duel, rmop);
    c.register_effect(duel, e5);
}

fn splimit(duel: &Duel, e: EffectId, se: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || (st & SUMMON_TYPE_SYNCHRO == SUMMON_TYPE_SYNCHRO && se.is_none())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, None);
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, RED_NOVA_DRAGON)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, None);
    if c.is_relate_to_effect(duel, e) && c.is_able_to_remove(duel) {
        g.insert(c);
    }
    if g.is_empty()
        || duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 0
        || !g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), 1, None)
    {
        return Ok(());
    }
    let sg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, None);
    if sg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    sc.set_material(duel, None);
    duel.break_effect()?;
    if duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}
