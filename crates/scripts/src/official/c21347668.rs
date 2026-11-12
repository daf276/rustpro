// Purrely Sleepy Memory
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 21347668;

// In the hand, and with Epurrely Noir on the field to attach it and with Sangan on the opponent's side to be turned.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (62592805, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesGranted(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_XMATERIAL + EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && a.e.get_handler(duel).is_set_card(duel, &[SET_PURRELY])
    });
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_level(duel, &[1]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.break_effect()?;
        if duel.discard_hand_full(tp, discardable, 1, 1, REASON_EFFECT | REASON_DISCARD, None)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    if r & (REASON_BATTLE | REASON_EFFECT) > 0 {
        duel.hint(HINT_CARD, 0, u64::from(ID));
        e.reset(duel);
        return 0;
    }
    val
}
