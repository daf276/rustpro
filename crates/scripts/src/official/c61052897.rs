// R-Genex Turing
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 61052897;

// In hand with Genex Controllers on the field for the condition and the Synchro Summon of Arms of Genex Return Zero.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (68505803, MZONE),
    (68505803, MZONE),
    (61775475, EXTRA),
    (61775475, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_synchro_material_operation(duel, synop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && duel.is_main_phase()
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_GENEX]),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GENEX]) && c.is_synchro_summonable(duel, None, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(scfilter, tp, LOCATION_EXTRA, 0, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.break_effect()?;
        duel.synchro_summon(tp, sc, None, None)?;
    }
    Ok(())
}

fn synop(duel: &Duel, e: EffectId, _: CardSet, _: CardSet, sg: CardSet, lv: u32, sc: CardId, _: u8) -> (bool, bool) {
    let c = e.get_handler(duel);
    let sum = sg.get_sum(duel, |m, duel| if m == c { 0 } else { i64::from(m.get_synchro_level(duel, sc)) });
    if sum + i64::from(c.get_synchro_level(duel, sc)) == i64::from(lv) {
        return (true, true);
    }
    (
        sc.is_set_card(duel, &[SET_GENEX]) && (sum + 1 == i64::from(lv) || sum + 3 == i64::from(lv)),
        true,
    )
}
