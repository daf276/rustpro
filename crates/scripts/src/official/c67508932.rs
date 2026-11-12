// Timelord Progenitor Vorpgate
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 67508932;

// In the Extra Deck, Synchro Summoned with Dragon Core Hexer (a Level 8 Tuner) and Doron (Level 2), to battle the opponent's Key Maces.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (38955728, MZONE),
    (38955728, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
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
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DAMAGE_STEP_END);
    e4.set_condition(duel, rmcond);
    e4.set_target(duel, rmtg);
    e4.set_operation(duel, rmop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_PHASE + PHASE_END);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID + 1) > 0);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn rmcond(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_location(duel) & LOCATION_EXTRA == LOCATION_EXTRA
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, LOCATION_MZONE as u8, i32::from(1 - tp));
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, None);
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0 {
        let fid = i64::from(c.get_field_id(duel));
        for oc in duel.get_operated_group() {
            oc.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, fid, 0);
        }
        c.register_flag_effect(duel, ID + 1, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_value(duel, HALF_DAMAGE as i32);
    duel.register_effect(e1, tp);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.get_flag_effect_label(duel, ID) == Some(i64::from(e.get_handler(duel).get_field_id(duel)))
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, LOCATION_REMOVED, None);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), g.len() as u8, tp, LOCATION_REMOVED as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(1 - tp, LOCATION_MZONE);
    let tg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, LOCATION_REMOVED, None);
    if ft <= 0 || tg.is_empty() {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = tg.select(duel, tp, ft as u8, ft as u8, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
