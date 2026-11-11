// Clock Tower Prison
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT};

const ID: u32 = 75041269;
const DREADMASTER: u32 = 40591390;
const COUNTER_CLOCK: u16 = 0x1b;

pub const LISTED_NAMES: &[u32] = &[DREADMASTER];

// Destiny HERO - Dreadmaster in the hand and Deck of each player to Special Summon, and another copy of the Spell.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (ID, OPPONENT),
    (DREADMASTER, DECK),
    (DREADMASTER, HAND),
    (DREADMASTER, DECK | OPPONENT),
    (DREADMASTER, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_CLOCK);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_COUNTER);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).add_counter(duel, COUNTER_CLOCK, 1);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_counter(duel, COUNTER_CLOCK) >= 4);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_LEAVE_FIELD_P);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_operation(duel, regop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_condition(duel, spcon);
    e5.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, { LOCATION_HAND | LOCATION_DECK }>,
    );
    e5.set_operation(duel, spop);
    e5.set_label_object(duel, Some(e4));
    c.register_effect(duel, e5);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let counted = a.e.get_handler(duel).get_counter(duel, COUNTER_CLOCK) >= 4;
    a.e.set_label(duel, counted.into());
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_reason(duel, REASON_DESTROY) && a.e.get_label_object(duel).is_some_and(|e| e.get_label(duel) == 1)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, DREADMASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 1, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
