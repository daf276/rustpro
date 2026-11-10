// Riryoku
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 34016756;

// In the hand, with Sangan on the field and Lucina on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE), (26202165, MZONE), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut g = duel.get_target_cards(e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    let Some(mut hc) = g.get_first() else { return Ok(()) };
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
        let Some(selected) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        hc = selected;
    }
    g.remove(hc);
    let c = e.get_handler(duel);
    let atk = hc.get_attack(duel);
    // Lua halves with a float, which the core rounds.
    let half = (f64::from(atk) / 2.0).round() as i32;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, half);
    hc.register_effect(duel, e1);
    if !hc.is_immune_to_effect(duel, e1)
        && let Some(other) = g.get_first()
    {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e2.set_value(duel, half);
        other.register_effect(duel, e2);
    }
    Ok(())
}
