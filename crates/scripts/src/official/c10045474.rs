// Infinite Impermanence
use crate::prelude::*;

const ID: u32 = 10045474;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, handcon);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable_monster(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    let c = a.e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    let set = a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && !c.is_status(duel, STATUS_ACT_FROM_HAND)
        && c.is_previous_position(duel, POS_FACEDOWN);
    duel.set_target_param(if set { POS_FACEDOWN } else { 0 });
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.is_can_be_disabled_by_effect(duel, e, true)) {
        return Ok(());
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    super::c97268402::negate(duel, c, tc, 0);
    let pos: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if c.is_relate_to_effect(duel, e) && pos & POS_FACEDOWN > 0 {
        duel.break_effect()?;
        c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 0);
        let seq = i64::from(c.get_sequence(duel));
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD);
        e3.set_code(duel, EFFECT_DISABLE);
        e3.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
        e3.set_target_card(duel, distg);
        e3.set_reset(duel, RESET_PHASE | PHASE_END);
        e3.set_label(duel, seq);
        duel.register_effect(e3, tp);
        let e4 = e3.clone_effect(duel);
        e4.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
        duel.register_effect(e4, tp);
        let e5 = duel.create_effect(c);
        e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e5.set_code(duel, EVENT_CHAIN_SOLVING);
        e5.set_operation(duel, disop);
        e5.set_reset(duel, RESET_PHASE | PHASE_END);
        e5.set_label(duel, seq);
        duel.register_effect(e5, tp);
        duel.hint(HINT_ZONE, tp, 1 << (seq + 8));
    }
    Ok(())
}

fn distg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let mut seq = e.get_label(duel);
    if c.is_controler(duel, 1 - e.get_handler_player(duel)) {
        seq = 4 - seq;
    }
    c.is_spell_trap(duel) && seq == i64::from(c.get_sequence(duel)) && c.get_flag_effect(duel, ID) == 0
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, ev) = (a.e, a.ev as u8);
    let mut cseq = e.get_label(duel);
    let Some(re) = a.re else { return Ok(()) };
    if !re.is_spell_trap_effect(duel) {
        return Ok(());
    }
    let rc = re.get_handler(duel);
    if rc.has_flag_effect(duel, ID) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CONTROLER);
    let mut loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    let mut seq: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_SEQUENCE);
    if re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        if loc & LOCATION_SZONE == 0 || rc.is_controler(duel, 1 - p) {
            seq = if rc.is_location(duel, LOCATION_SZONE) && rc.is_controler(duel, p) {
                rc.get_sequence(duel)
            } else {
                rc.get_previous_sequence(duel)
            };
        }
        // GetValue pushes 0, never nil, for an effect without a value.
        if loc & LOCATION_SZONE == 0
            && let Value::Int(val) = re.get_value(duel)
            && [LOCATION_SZONE, LOCATION_FZONE, LOCATION_PZONE].contains(&(val as u32))
        {
            loc = LOCATION_SZONE;
        }
    }
    if a.ep != e.get_handler_player(duel) {
        cseq = 4 - cseq;
    }
    if loc & LOCATION_SZONE != 0 && cseq == i64::from(seq) {
        duel.negate_effect(ev);
    }
    Ok(())
}

fn handcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_ONFIELD, 0) == 0
}
