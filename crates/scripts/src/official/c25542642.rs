// Phantom Knights' Fog Blade
use engine::constants::MSG_CARD_TARGET;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 25542642;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS];
// Set, to be activated on the opponent's Effect Monster Elfnote Fortuna; Doomed Soleret in the GY to revive.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, GRAVE), (85976588, MZONE | OPPONENT), (83566725, GRAVE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CARD_TARGET), Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl PersistentProcedure for Procedure {
    const CATEGORY: u64 = CATEGORY_DISABLE;
    const HINT2: Option<u32> = Some(TIMINGS_CHECK_MONSTER);

    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
    }

    fn tg(duel: &mut Duel, _: &Args, tc: CardId) -> Script<()> {
        duel.set_operation_info(0, CATEGORY_DISABLE, Some(tc.into()), 1, 0, 0);
        Ok(())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Persistent::add_procedure::<Procedure>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, Persistent::target_filter);
    c.register_effect(duel, e3);
    let e5 = e3.clone_effect(duel);
    e5.set_code(duel, EFFECT_DISABLE);
    c.register_effect(duel, e5);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_value(duel, Value::Card(|duel, e, c| i32::from(Persistent::target_filter(duel, e, c))));
    c.register_effect(duel, e4);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e6.set_range(duel, LOCATION_SZONE);
    e6.set_code(duel, EVENT_LEAVE_FIELD);
    e6.set_condition(duel, descon);
    e6.set_operation(duel, desop);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 0));
    e7.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e7.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e7.set_type(duel, EFFECT_TYPE_QUICK_O);
    e7.set_code(duel, EVENT_FREE_CHAIN);
    e7.set_range(duel, LOCATION_GRAVE);
    e7.set_count_limit(duel, 1, ID);
    e7.set_cost(duel, Cost::self_banish);
    e7.set_target(duel, sptg);
    e7.set_operation(duel, spop);
    e7.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e7);
}

pub fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if c.is_status(duel, STATUS_DESTROY_CONFIRMED) {
        return false;
    }
    c.get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.is_contains(tc)))
}

pub fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.destroy(a.e.get_handler(duel), REASON_EFFECT)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

// Shared with Phantom Knights' Wing.
pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
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

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        tc.register_effect_full(duel, e1, true);
    }
    Ok(())
}
