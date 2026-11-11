// Heavy Armored Knight Babeldecker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, OPPONENT, SZONE};

const ID: u32 = 45116390;

// In the hand to Normal Summon without Tributing, with EARTH Machine monsters (Overdrive, Golgoil) to Special Summon from the
// hand, the Rank 10 EARTH Machine Xyz Monsters Gustav Max and Super Dora in the Extra Deck, and the opponent's Ash Blossom and
// Infinite Impermanence to chain.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (2311603, HAND),
    (7526150, HAND),
    (56910167, EXTRA),
    (49032236, EXTRA),
    (14558127, HAND | OPPONENT),
    (10045474, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 0));
    e2a.set_target(duel, handsptg);
    e2a.set_operation(duel, handspop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| duel.get_custom_activity_count(ID, 1 - a.tp, ACTIVITY_CHAIN) > 0);
    e3.set_target(duel, extrasptg);
    e3.set_operation(duel, extraspop);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|_, _, _, _| false));
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && c.is_level_above(duel, 4) && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn handspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_race(duel, RACE_MACHINE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| handspfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| handspfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn extraspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && c.is_rank(duel, &[10])
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.is_race(duel, RACE_MACHINE)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn extrasptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(|sc, duel| extraspfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn extraspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || c.is_controler(duel, 1 - tp) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if pg.len() > 1 || (pg.len() == 1 && !pg.contains(c)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| extraspfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(xc) = g.get_first() else { return Ok(()) };
    xc.set_material(duel, Some(c.into()));
    duel.overlay(xc, c)?;
    if duel.special_summon(xc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        xc.complete_procedure(duel);
    }
    Ok(())
}
