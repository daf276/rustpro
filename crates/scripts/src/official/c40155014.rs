// Centur-Ion Phalanx
use crate::keys;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 40155014;

pub const LISTED_SERIES: &[u16] = &[SET_CENTUR_ION];
// Centur-Ion Legatia and Auxila in the Graveyard to Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, GRAVE), (15982593, GRAVE), (71858682, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn rmvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_original_type(duel, TYPE_MONSTER) && c.is_able_to_remove(duel)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && rmvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmvfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmvfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_REMOVED as i32);
    Ok(false)
}

struct Return;

impl aux::DelayedOperation for Return {
    fn operate(duel: &mut Duel, ag: CardSet, a: &Args) -> Script<()> {
        let owner = ag.get_first().map_or(PLAYER_NONE, |tc| tc.get_owner(duel));
        duel.special_summon(ag, 0, a.tp, owner, false, false, POS_FACEUP)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(keys::delayed_turn(ID, a.e.get_label(duel))) + 1
    }
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0 && tc.is_location(duel, LOCATION_REMOVED) {
        let turn_ct = i64::from(duel.get_turn_count());
        let reset_ct = if duel.get_current_phase() <= PHASE_STANDBY { 2 } else { 1 };
        let reset = RESET_PHASE | PHASE_STANDBY;
        if let Some(delayed) = aux::delayed_operation_full::<Return>(duel, tc.into(), PHASE_STANDBY, reset, ID, e, tp, 0, reset_ct) {
            let key = keys::delayed_turn(ID, delayed.get_label(duel));
            duel.set_script_state(key, turn_ct);
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_type(duel, TYPE_SYNCHRO) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -1500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}
