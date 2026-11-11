// Mecha Phantom Beast Warbluran
use super::c72291078::{indcon, token_stats};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 31480215;

pub const LISTED_NAMES: &[u32] = &[TOKEN_MECHA_PHANTOM_BEAST];

// On the field and in the hand, with Mecha Phantom Beast O-Lion to release, and Armory Arm in the Extra Deck to Synchro Summon
// with Petit Angel (Level 3).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (72291078, HAND),
    (29071332, EXTRA),
    (29071332, EXTRA),
    (29071332, EXTRA),
    (38142739, MZONE),
    (38142739, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_MECHA_PHANTOM_BEAST),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, lvcost);
    e4.set_operation(duel, lvop);
    c.register_effect(duel, e4);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO && c.get_reason_card(duel).is_some_and(|rc| rc.is_race(duel, RACE_MACHINE))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_attribute(duel, ATTRIBUTE_WIND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, TOKEN_MECHA_PHANTOM_BEAST, token_stats())
        && let Some(token) = duel.create_token(tp, TOKEN_MECHA_PHANTOM_BEAST)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn lvcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let filter = |tc: CardId, duel: &Duel| tc.is_set_card(duel, &[SET_MECHA_PHANTOM_BEAST]);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
