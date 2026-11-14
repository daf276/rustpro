// Prologue of the Destruction Swordsman
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 33280639;
const BUSTER_DRAGON: u32 = 11790356;

pub const LISTED_SERIES: &[u16] = &[SET_DESTRUCTION_SWORD, SET_BUSTER_BLADER];
pub const LISTED_NAMES: &[u32] = &[ID, BUSTER_DRAGON];

// In hand with Buster Whelp and Buster Blader in the Deck to send and Buster Dragon in the Extra Deck, and in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (49823708, DECK),
    (49823708, DECK),
    (CARD_BUSTER_BLADER, DECK),
    (CARD_BUSTER_BLADER, DECK),
    (BUSTER_DRAGON, EXTRA),
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
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_condition(duel, |duel, a| !duel.has_flag_effect(a.tp, ID));
    e2.set_operation(duel, immop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_DESTRUCTION_SWORD]) && !c.is_code(duel, ID))
        || (c.is_set_card(duel, &[SET_BUSTER_BLADER]) && c.is_monster(duel)))
        && c.is_able_to_grave_as_cost(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let ok = sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_DESTRUCTION_SWORD]), 1, None)
        && sg.is_exists(
            duel,
            |c, duel| c.is_set_card(duel, &[SET_BUSTER_BLADER]) && c.is_monster(duel),
            1,
            None,
        );
    (ok, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(cfilter, tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let sg = aux::select_unselect_group_full(duel, g, 2, 2, &rescon, tp, HINTMSG_TOGRAVE, Some(&rescon), false)?;
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_code(duel, BUSTER_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_EXTRA | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
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
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let turn = i64::from(duel.get_turn_count());
        if let Some(delayed) = aux::delayed_operation_full::<Destroy>(duel, sc.into(), PHASE_END, RESET_PHASE | PHASE_END, ID, e, tp, 0, 2)
        {
            let key = crate::keys::delayed_turn(ID, delayed.get_label(duel));
            duel.set_script_state(key, turn);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn immop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, LOCATION_ONFIELD, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_DESTRUCTION_SWORD]));
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    duel.register_effect(e2, tp);
    Ok(())
}
