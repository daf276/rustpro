// Future Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 77565204;

// Fusionist in the Extra Deck, with its materials Petit Angel and Mystical Sheep #2 in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (1641882, EXTRA), (38142739, DECK), (83464209, DECK)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(1641882)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, reg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, tgcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, proccon);
    e3.set_operation(duel, procop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_code(duel, EVENT_LEAVE_FIELD);
    e5.set_condition(duel, descon2);
    e5.set_operation(duel, desop2);
    c.register_effect(duel, e5);
}

fn reg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let c = e.get_handler(duel);
    c.set_turn_counter(duel, 0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE_START | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_operation(duel, ctop);
    duel.register_effect(e1, tp);
    c.create_effect_relation(duel, e1);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ct = c.get_turn_counter(duel);
    if !c.is_relate_to_effect(duel, e) || ct >= 2 {
        c.set_turn_counter(duel, 0);
        e.reset(duel);
        return Ok(());
    }
    if duel.is_turn_player(1 - tp) {
        return Ok(());
    }
    c.set_turn_counter(duel, ct + 1);
    Ok(())
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_turn_counter(duel) == 1
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn filter1(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_monster(duel) && c.is_able_to_grave(duel) && !c.is_immune_to_effect(duel, e)
}

fn filter2(c: CardId, duel: &Duel, m: CardSet) -> bool {
    c.is_fusion_summonable_card(duel) && c.check_fusion_material(duel, Some(m), None, u64::from(PLAYER_NONE))
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = duel.get_matching_group(|c, duel| filter1(c, duel, e), tp, LOCATION_DECK, 0, None);
    let sg = duel.get_matching_group(|c, duel| filter2(c, duel, mg), tp, LOCATION_EXTRA, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let tg = sg.select(duel, tp, 1, 1, None)?;
    let Some(tc) = tg.get_first() else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, tc)?;
    let code = tc.get_code(duel);
    let mat = duel.select_fusion_material(tp, tc, mg, None, u64::from(PLAYER_NONE))?;
    duel.send_to_grave(mat, REASON_EFFECT)?;
    for mc in mat {
        mc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    }
    e.set_label(duel, i64::from(code));
    e.set_label_object_group(duel, Some(mat));
    Ok(())
}

fn proccon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_turn_counter(duel) == 2
}

fn procfilter(c: CardId, duel: &Duel, code: u32, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, code)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
}

fn procop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(e2) = e.get_label_object(duel) else {
        return Ok(());
    };
    let code = e2.get_label(duel) as u32;
    let Some(mat) = e2.get_label_object_group(duel).filter(|mat| !mat.is_empty()) else {
        return Ok(());
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| procfilter(c, duel, code, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else {
        return Ok(());
    };
    tc.set_status(duel, STATUS_FUTURE_FUSION, true);
    for mc in mat {
        if mc.get_flag_effect(duel, ID) > 0 {
            mc.set_reason(duel, REASON_EFFECT + REASON_FUSION + REASON_MATERIAL);
        }
    }
    tc.set_material(duel, Some(mat));
    duel.special_summon(tc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)?;
    tc.complete_procedure(duel);
    tc.set_status(duel, STATUS_FUTURE_FUSION, false);
    c.set_card_target(duel, tc);
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_first_card_target(duel)
        && tc.is_location(duel, LOCATION_MZONE)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon2(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.is_contains(tc)) && tc.is_reason(duel, REASON_DESTROY))
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.destroy(a.e.get_handler(duel), REASON_EFFECT)?;
    Ok(())
}
