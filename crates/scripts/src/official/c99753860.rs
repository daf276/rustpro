// Ars Magna of Unification and Separation
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 99753860;

// Ars Magna monsters in the Graveyard to Special Summon and banish for the draw, a second copy banished to be Special Summoned when
// Purification Power Patron is Link Summoned from the Extra Deck with Dark Magician (Normal Monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, REMOVED),
    (62368221, GRAVE),
    (36270527, GRAVE),
    (31822037, MZONE),
    (31822037, EXTRA),
    (31822037, EXTRA),
    (46986414, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (37279096, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_REMOVED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default().is_exists(
            duel,
            aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_SYNCHRO | TYPE_LINK)),
            1,
            None,
        )
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 2));
    e3a.set_category(duel, CATEGORY_DRAW);
    e3a.set_type(duel, EFFECT_TYPE_IGNITION);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_count_limit(duel, 1, (ID, 2));
    e3a.set_condition(duel, |duel, a| !citrinitascon(duel, a));
    e3a.set_cost(duel, drawcost);
    e3a.set_target(duel, stock::draw_target::<1>);
    e3a.set_operation(duel, stock::draw);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3b.set_code(duel, EVENT_FREE_CHAIN);
    e3b.set_condition(duel, citrinitascon);
    e3b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3b);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ARS_MAGNA])
        && c.is_race_except(duel, RACE_SPELLCASTER)
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_GRAVE | LOCATION_REMOVED,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let c = e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel)
    });
    e1.set_value(duel, 2);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn citrinitascon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, EFFECT_ARS_MAGNA_CITRINITAS).is_some()
}

fn drawcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARS_MAGNA]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn drawcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, drawcostfilter, LOCATION_MZONE | LOCATION_GRAVE)
}
