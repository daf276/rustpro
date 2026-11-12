// T.G. Glaive Blaster
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 95973569;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with T.G. Mighty Striker (a Level 2 Tuner) and two T.G. Hyper Librarians, with its own
// copies to banish and Special Summon back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (32480825, MZONE),
    (90953320, MZONE),
    (90953320, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CUSTOM + ID);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    e3.set_label_object_group(duel, Some(CardSet::default()));
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_REMOVE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_label_object(duel, Some(e3));
    e4.set_operation(duel, regop);
    c.register_effect(duel, e4);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let ct = c
        .get_material(duel)
        .filter_count(duel, |mc, duel| !mc.is_type(duel, TYPE_TUNER), None);
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, ct as i64);
    }
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_synchro_summoned(duel) && !c.has_flag_effect_full(duel, ID, a.e.get_label(duel) as usize)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_special_summoned(duel) && c.is_summon_location(duel, LOCATION_EXTRA) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
        && c.is_monster(duel)
        && c.is_faceup(duel)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_REMOVED)
        && !c.is_type(duel, TYPE_TOKEN)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e3) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e3.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e3.set_label_object_group(duel, Some(g));
    duel.raise_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    Ok(())
}

// The Lua's `chkc` check calls the undefined `s.cfilter`, which errors, so it fails.
fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && !g.is_empty());
    }
    duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        g.select(duel, tp, 1, 1, None)?.get_first()
    } else {
        g.get_first()
    };
    if let Some(tc) = tc {
        duel.set_target_card(tc);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, tc.map(CardSet::from), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, a.tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
