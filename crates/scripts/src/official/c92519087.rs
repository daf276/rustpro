// Virtual World Kyubi - Shenshen
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 92519087;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned with Tune Warrior and Luster Dragon #2 (Level 6 Normal Monster), and a second copy in the Graveyard with
// two Normal Monsters of different Types and Attributes to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (74093656, MZONE),
    (17658803, MZONE),
    (55444629, GRAVE),
    (20624263, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| duel.is_player_can_remove(e.get_handler_player(duel), Some(c)));
    e1.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, tgcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, aux::exccon);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker().is_some_and(|tg| tg.is_controler(duel, a.tp))
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_monster),
            a.tp,
            LOCATION_REMOVED,
            LOCATION_REMOVED,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_ALL, LOCATION_REMOVED as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        aux::faceup_filter(CardId::is_monster),
        tp,
        LOCATION_REMOVED,
        LOCATION_REMOVED,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT | REASON_RETURN)?;
    }
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| c.get_original_race(duel) as i64) == sg.len()
            && sg.get_class_count(duel, |c, duel| i64::from(c.get_original_attribute(duel))) == sg.len(),
        false,
    )
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(rmfilter, tp, LOCATION_GRAVE | LOCATION_MZONE, 0, Some(a.e.get_handler(duel).into()));
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let g = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_REMOVE)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}
