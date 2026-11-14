// Primite Howl
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, SZONE};

const ID: u32 = 41488249;

// Set, with Mystical Elf (Normal Monster) in the Deck to declare, and in the GY with Mystical Elf there to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (15025844, DECK), (ID, GRAVE), (15025844, GRAVE)];
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
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, controltg);
    e2.set_operation(duel, controlop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let code = duel.announce_card(tp, &[u64::from(TYPE_NORMAL), OPCODE_ISTYPE])?;
    duel.set_target_param(code);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_label(duel, i64::from(code));
    e1.set_target_card(duel, |duel, e, c| {
        c.is_set_card(duel, &[SET_PRIMITE]) || (c.is_code(duel, e.get_label(duel) as u32) && c.is_type(duel, TYPE_NORMAL))
    });
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    if duel.get_field_group_count(tp, LOCATION_MZONE, 0) > 0 || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, code));
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel);
    c.is_type(duel, TYPE_NORMAL)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|c, duel| controlfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None)
}

fn controlfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_controler_can_be_changed(duel) && c.is_faceup(duel) && c.get_attack(duel) > atk
}

fn controltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = |c, duel: &Duel| tgfilter(c, duel, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, tp, 0);
    Ok(false)
}

fn controlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let atk = tc.get_attack(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_matching_card(tp, |c, duel| controlfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        let reason_player = duel.get_reason_player();
        duel.get_control_full(g, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
    }
    Ok(())
}
