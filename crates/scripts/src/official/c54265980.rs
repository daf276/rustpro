// DoomZ VII Seven - Elara
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 54265980;

// On the field, with Axe of Despair in the hand to equip, DoomZ Change and DoomZ Raiders in the Deck to Set, Dark Hole to
// destroy it, and DoomZ Break - Diactorus in the Extra Deck to attach it to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (40619825, HAND),
    (40619825, HAND),
    (53129443, HAND),
    (38007649, DECK),
    (38007649, DECK),
    (32442017, DECK),
    (31010081, EXTRA),
    (31010081, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SET);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, settg);
    e1a.set_operation(duel, setop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e1c = e1a.clone_effect(duel);
    e1c.set_code(duel, EVENT_DESTROYED);
    e1c.set_condition(duel, |_, a| a.r & REASON_EFFECT > 0);
    c.register_effect(duel, e1c);
    add_xyz_effect(duel, c, ID);
}

// The effect DoomZ VII, V and XII share: Xyz Summon over itself and what is equipped to it.
pub fn add_xyz_effect(duel: &mut Duel, c: CardId, id: u32) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(id, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (id, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_count(duel) > 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32, mc: CardId) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND)
        && c.is_race(duel, RACE_MACHINE)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_rank(duel, &[lv])
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
        && mc
            .get_equip_group(duel)
            .filter_count(duel, |ec, duel| ec.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT), None)
            == mc.get_equip_count(duel)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let lv = c.get_level(duel);
        return Ok(duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, lv, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = c.get_level(duel);
    let filter = |sc, duel: &Duel| spfilter(sc, duel, e, tp, lv, c);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
        let mut materials = CardSet::from(c);
        materials.merge(c.get_equip_group(duel));
        duel.overlay_full(sc, materials, true)?;
    }
    Ok(())
}
