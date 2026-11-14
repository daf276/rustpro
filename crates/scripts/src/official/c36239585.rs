// Ghostrick Fairy
use super::c80885284::{posop, postg, sumcon};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT};

const ID: u32 = 36239585;

pub const LISTED_SERIES: &[u16] = &[SET_GHOSTRICK];

const GHOSTRICK_JIANGSHI: u32 = 80885284;
const GHOSTRICK_SCARE: u32 = 86516889;
const ELFNOTE_LUCINA: u32 = 13597785;

pub const RESET_FLAG_STANDARD: u32 = RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET) | RESET_PHASE | PHASE_END;

// Face-down copies with an attacker of the opponent's to flip them, and Ghostrick Jiangshi and Scare in the Graveyard to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (GHOSTRICK_JIANGSHI, GRAVE),
    (GHOSTRICK_SCARE, GRAVE),
    (GHOSTRICK_JIANGSHI, DECK),
    (ELFNOTE_LUCINA, MZONE | OPPONENT),
    (GHOSTRICK_JIANGSHI, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_condition(duel, sumcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, postg::<ID, RESET_FLAG_STANDARD>);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_FLIP);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !c.is_set_card(duel, &[SET_GHOSTRICK]) {
        return false;
    }
    if c.is_monster(duel) {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
    } else if c.is_spell_trap(duel) {
        (c.is_field_spell(duel) || duel.get_location_count(tp, LOCATION_SZONE) > 0) && c.is_ssetable(duel)
    } else {
        false
    }
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && setfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| setfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, |c, duel| setfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first()) {
        if tc.is_monster(duel) {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
        } else if tc.is_spell_trap(duel) {
            e.set_category(duel, CATEGORY_SET);
            duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, LOCATION_GRAVE as i32);
        }
    }
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, 1 - tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let mut andifyoudo = false;
    if tc.is_monster(duel) {
        andifyoudo =
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)? > 0;
    } else if tc.is_spell_trap(duel) {
        if tc.is_field_spell(duel)
            && let Some(fc) = duel.get_field_card(tp, LOCATION_FZONE, 0)
        {
            duel.send_to_grave(fc, REASON_RULE)?;
            duel.break_effect()?;
        }
        if tc.is_field_spell(duel) || duel.get_location_count(tp, LOCATION_SZONE) > 0 {
            duel.sset(tp, tc)?;
            andifyoudo = true;
        }
    }
    if andifyoudo {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        tc.register_effect(duel, e1);
        if duel.is_existing_matching_card(CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.break_effect()?;
            let ct = duel.get_matching_group_count(CardId::is_facedown, tp, LOCATION_ONFIELD, 0, None);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let g = duel.select_matching_card(tp, CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, ct as u8, None)?;
            duel.hint_selection(g);
            duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}
