// Selene, Queen of the Master Magicians
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, SZONE};
use engine::constants::MSG_ADD_COUNTER;

const ID: u32 = 45819647;

pub const LISTED_SERIES: &[u16] = &[SET_ENDYMION];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_SPELL];

// Dark Magician (a Spellcaster) as the Link Materials and to Special Summon, and Spells for the counters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_DARK_MAGICIAN, GRAVE),
    (5318639, GRAVE),
    (5318639, GRAVE),
    (5318639, SZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::Message(MSG_ADD_COUNTER)];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        lcheck(duel, sg, lc, sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_SPELL);
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ENDYMION]));
        duel.is_existing_matching_card(filter, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
    });
    e2.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, _| duel.is_main_phase());
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn lcheck(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
    g.is_exists(
        duel,
        |c, duel| c.is_race_full(duel, RACE_SPELLCASTER, Some(lc), sumtype, tp),
        1,
        None,
    )
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel)
        && (c.is_location(duel, LOCATION_GRAVE)
            || c.is_faceup(duel)
            || c.get_equip_target(duel).is_some()
            || c.is_location(duel, LOCATION_FZONE))
}

fn count(duel: &Duel, tp: u8) -> usize {
    let locations = LOCATION_ONFIELD | LOCATION_GRAVE;
    duel.get_matching_group_count(ctfilter, tp, locations, locations, None)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ct = count(duel, a.tp);
    if ct > 0 {
        duel.set_operation_info(0, CATEGORY_COUNTER, None, ct as u8, 0, i32::from(COUNTER_SPELL));
    }
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = count(duel, a.tp);
    if c.is_relate_to_effect(duel, a.e) && ct > 0 {
        c.add_counter(duel, COUNTER_SPELL, ct as u16);
    }
    Ok(())
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_can_remove_counter(tp, 1, 0, COUNTER_SPELL, 3, REASON_COST));
    }
    duel.remove_counter(tp, 1, 0, COUNTER_SPELL, 3, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
        return Ok(zone != 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, zone),
                tp,
                LOCATION_HAND | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
    if zone == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, zone));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon_full(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE, zone)?;
    }
    Ok(())
}
