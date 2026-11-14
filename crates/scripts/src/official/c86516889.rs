// Ghostrick Scare
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 86516889;

pub const LISTED_SERIES: &[u16] = &[SET_GHOSTRICK];

const GHOSTRICK_JIANGSHI: u32 = 80885284;

// Set in the Spell and Trap Zone with face-down Ghostrick monsters to flip and the opponent's monsters to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_JIANGSHI, MZONE | OPPONENT),
    (GHOSTRICK_JIANGSHI, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_facedown(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_facedown, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEDOWN);
    let g = duel.select_target(tp, CardId::is_facedown, tp, LOCATION_MZONE, 0, 1, 99, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_facedown(duel) && c.is_relate_to_effect(duel, e)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tg) = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    let g = tg.filter(duel, |c, duel| filter(c, duel, e), None);
    duel.change_position(g, POS_FACEUP_DEFENSE)?;
    let ct = g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_GHOSTRICK]), None);
    if ct > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let sg = duel.select_matching_card(tp, CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, ct as u8, None)?;
        if !sg.is_empty() {
            duel.hint_selection(sg);
            duel.change_position(sg, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}
