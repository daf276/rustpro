// K9-EW Special Release Experiment
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, SZONE};

const ID: u32 = 26585784;

pub const LISTED_SERIES: &[u16] = &[SET_K9];
// Set, with K9-17 Izuna and K9-ØØ Lupis in the hand and Graveyard to summon, K9-17 "Ripper" and K9-X "Werewolf" in the
// Extra Deck to Xyz Summon over them, and copies and K9-X Forced Release in the Graveyard to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (92248362, HAND),
    (91025875, HAND),
    (92248362, GRAVE),
    (27420823, EXTRA),
    (90303227, EXTRA),
    (53792930, GRAVE),
    (53792930, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, TIMING_END_PHASE, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_phase(PHASE_END));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_K9]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn exspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_K9])
        && c.is_type(duel, TYPE_XYZ)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

struct Destroy;

impl aux::DelayedOperation for Destroy {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.destroy(ag, REASON_EFFECT)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(crate::keys::delayed_turn(ID, a.e.get_label(duel))) + 1
    }
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(sc.into()), tp, None, None, REASON_XYZ);
    if (pg.is_empty() || (pg.len() == 1 && pg.contains(sc)))
        && duel.is_existing_matching_card(|c, duel| exspfilter(c, duel, e, tp, sc), tp, LOCATION_EXTRA, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| exspfilter(c, duel, e, tp, sc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(xyzc) = g.get_first() {
            xyzc.set_material(duel, Some(sc.into()));
            duel.overlay(xyzc, sc)?;
            duel.break_effect()?;
            if duel.special_summon(xyzc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? == 0 {
                return Ok(());
            }
            xyzc.complete_procedure(duel);
            let turn = i64::from(duel.get_turn_count());
            let reset = RESET_PHASE | PHASE_END;
            if let Some(delayed) =
                aux::delayed_operation_full::<Destroy>(duel, xyzc.into(), PHASE_END, reset, ID, e, tp, aux::stringid(ID, 3), 2)
            {
                let key = crate::keys::delayed_turn(ID, delayed.get_label(duel));
                duel.set_script_state(key, turn);
            }
        }
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_K9]) && c.is_quick_play_spell(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && setfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(setfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, setfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.sset(a.tp, tc)?;
    }
    Ok(())
}
