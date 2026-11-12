// Dark Dimension Soldier
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, REMOVED, SZONE};

const ID: u32 = 109401;

// In the Extra Deck, Synchro Summoned from Water Spirit (Level 1 Tuner) and Prevent Rat (Level 4), with DARK monsters banished
// and Set Spells/Traps for the End Phase damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (13193642, REMOVED),
    (13193642, REMOVED),
    (5318639, SZONE),
    (5318639, SZONE),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, (ID, 1));
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 2));
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_monster(duel)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon(tc, 0, tp, tp, false, false, POS_DEFENSE)? > 0 && tc.is_facedown(duel) {
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let ct = duel.get_matching_group_count(CardId::is_facedown, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ct as i32 * 100);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = duel.get_matching_group_count(CardId::is_facedown, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.damage(p, ct as u32 * 100, REASON_EFFECT)?;
    Ok(())
}
