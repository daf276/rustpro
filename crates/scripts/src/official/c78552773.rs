// Supay
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 78552773;

pub const LISTED_NAMES: &[u32] = &[42280216];

// Copies on the field destroyed by the opponent's Raigeki, with Oracle of the Sun in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (42280216, DECK),
    (42280216, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(42280216)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_reason(duel, REASON_EFFECT)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 42280216) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, tc.get_attack(duel) * 2);
        tc.register_effect_full(duel, e1, true);
        duel.special_summon_complete()?;
        let e2 = duel.create_effect(e.get_handler(duel));
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_code(duel, EVENT_PHASE + PHASE_END);
        e2.set_operation(duel, retop);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e2.set_count_limit(duel, 1, 0);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.send_to_hand(a.e.get_handler(duel), PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
