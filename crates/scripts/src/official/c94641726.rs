// Storm-Bane Dragon Destorbim
use crate::prelude::*;

const ID: u32 = 94641726;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_REMOVE);
    e2a.set_type(duel, EFFECT_TYPE_IGNITION);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit(duel, 1, (ID, 0));
    e2a.set_condition(duel, |duel, a| {
        let tp = a.tp;
        duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) <= duel.get_field_group_count(tp, LOCATION_ONFIELD, 0)
    });
    e2a.set_cost(duel, bancost);
    e2a.set_target(duel, bantg);
    e2a.set_operation(duel, banop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2b.set_code(duel, EVENT_FREE_CHAIN);
    e2b.set_condition(duel, |duel, a| {
        let tp = a.tp;
        duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > duel.get_field_group_count(tp, LOCATION_ONFIELD, 0)
    });
    e2b.set_hint_timing(
        duel,
        TIMING_SPSUMMON,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn bancostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && !c.is_summonable_card(duel) && c.is_able_to_remove_as_cost(duel)
}

fn removable(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel)
}

fn bancost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(bancostfilter, tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(removable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let max_count = duel.get_matching_group_count(removable, tp, 0, LOCATION_ONFIELD, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, bancostfilter, tp, LOCATION_GRAVE, 0, 1, max_count as u8, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    a.e.set_label(duel, g.len() as i64);
    Ok(false)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.e.get_label(duel) as u8;
    duel.set_operation_info(0, CATEGORY_REMOVE, None, count, 1 - a.tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let banish_count = a.e.get_label(duel) as usize;
    if duel.get_matching_group_count(removable, tp, 0, LOCATION_ONFIELD, None) < banish_count {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let count = banish_count as u8;
    let g = duel.select_matching_card(tp, removable, tp, 0, LOCATION_ONFIELD, count, count, None)?;
    if g.len() == banish_count {
        duel.hint_selection(g);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_REMOVED)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_REMOVED, POS_FACEUP)
}
