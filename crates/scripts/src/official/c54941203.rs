// Tuning Magician
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, PZONE};

const ID: u32 = 54941203;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICIAN];

const STARGAZER_MAGICIAN: u32 = 94415058;

// Two Magician scales (Stargazer Magician) to Special Summon it from the hand and the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (STARGAZER_MAGICIAN, PZONE), (STARGAZER_MAGICIAN, PZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_count_limit(duel, 1, ID);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target(duel, rectg);
    e2.set_operation(duel, recop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(|c, duel| c.is_set_card(duel, &[SET_MAGICIAN]), a.tp, LOCATION_PZONE, 0, 2, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, 1 - tp, 400);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 400);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    if duel.recover(p, 400, REASON_EFFECT)? != 0 {
        duel.break_effect()?;
        duel.damage(a.tp, 400, REASON_EFFECT)?;
    }
    Ok(())
}
