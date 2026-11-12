// Laevatein, Generaider Boss of Shadows
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 74615388;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned with Level 9 monsters (Wandering Titan of Tartarus, Thunder King, the Lightningstrike Kaiju), beside Jormungandr
// in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2665273, EXTRA),
    (43363035, MZONE),
    (43363035, MZONE),
    (48770333, MZONE),
    (48770333, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, -1000);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_releasable(duel));
    }
    duel.set_target_param(c.get_overlay_count(duel) as u32);
    duel.release(c, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ec: CardId) -> bool {
    c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_GENERAIDER])
        && !c.is_race(duel, RACE_FAIRY)
        && duel.get_location_count_from_ex(tp, tp, Some(ec.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ec = e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, ec), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn ovfilter(c: CardId, duel: &Duel, xc: CardId, tp: u8, e: EffectId) -> bool {
    c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT) && !c.is_immune_to_effect(duel, e)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |tc, duel| spfilter(tc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let ct: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if ct == 0 {
        return Ok(());
    }
    let og = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| ovfilter(c, duel, tc, tp, e)),
        tp,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        Some(tc.into()),
    );
    if og.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = og.select(duel, tp, 1, ct as u8, None)?;
    if !g.is_empty() {
        for mc in g {
            if mc.is_spell_trap(duel) && mc.is_location(duel, LOCATION_SZONE) {
                mc.cancel_to_grave(duel);
            }
        }
        duel.hint_selection(g);
        duel.break_effect()?;
        duel.overlay_full(tc, g, true)?;
    }
    Ok(())
}
