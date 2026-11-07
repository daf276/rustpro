// Number 90: Galaxy-Eyes Photon Lord
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 8165596;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Stealth Angecradle, with Galaxy Serpent in the Deck to search or attach.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (64413994, MZONE), (64413994, MZONE), (11066358, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_PHOTON]), 1, None)
    });
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
    });
    e2.set_cost(duel, |duel, a, chk| {
        Cost::detach_from_self_op::<1, 1>(duel, a, chk, |duel, e, og| {
            let galaxy = og.get_first().is_some_and(|c| c.is_set_card(duel, &[SET_GALAXY]));
            e.set_label(duel, galaxy.into());
        })
    });
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, thmattg);
    e3.set_operation(duel, thmatop);
    c.register_effect(duel, e3);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) && e.get_label(duel) == 1 {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_effect(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) && a.e.get_label(duel) == 1 {
        duel.break_effect()?;
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn thmatfilter(c: CardId, duel: &Duel, hc: CardId, tp: u8, relation_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_PHOTON, SET_GALAXY])
        && (c.is_able_to_hand(duel) || (relation_chk && c.is_can_be_xyz_material(duel, Some(hc), tp, REASON_EFFECT)))
}

fn thmattg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(|sc, duel| thmatfilter(sc, duel, c, tp, true), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thmatop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let relation_chk = c.is_relate_to_effect(duel, e);
    let desc = if relation_chk { aux::stringid(ID, 2) } else { HINTMSG_ATOHAND };
    duel.hint(HINT_SELECTMSG, tp, desc);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| thmatfilter(sc, duel, c, tp, relation_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| relation_chk && sc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
        |duel| duel.overlay(c, sc),
        aux::stringid(ID, 3),
    )
}
