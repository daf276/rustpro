// Mementotlan Cranium Burst
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 79600447;

pub const LISTED_SERIES: &[u16] = &[SET_MEMENTO];
pub const LISTED_NAMES: &[u32] = &[CARD_MEMENTOAL_TECUHTLICA];

// Set, with Mementoal Tecuhtlica on the field and the opponent's Mementotlan Angwitch activating its effects on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (CARD_MEMENTOAL_TECUHTLICA, MZONE),
    (18165869, MZONE),
    (54550967, MZONE | OPPONENT),
    (54550967, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_hint_timing(duel, TIMING_ATTACK, TIMINGS_CHECK_MONSTER_E | TIMING_ATTACK);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_MUST_ATTACK);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_condition(duel, atcon);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e2.set_value(duel, Value::OptionalCard(atlimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_condition(duel, negcon);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn memento_filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MEMENTO])
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(memento_filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

// The Lua errors in IsContains for the nil card the core passes when it checks the attacker alone, which reads as 0.
fn atlimit(duel: &Duel, e: EffectId, c: Option<CardId>) -> i32 {
    let Some(c) = c else { return 0 };
    let g = duel.get_matching_group(memento_filter, e.get_handler_player(duel), LOCATION_MZONE, 0, None);
    i32::from(
        g.get_max_group(duel, |c, duel| c.get_attack(duel).into())
            .is_some_and(|(mg, _)| mg.is_contains(c)),
    )
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    a.re.is_some_and(|re| re.is_monster_effect(duel)) && loc & LOCATION_MZONE != 0 && a.ep == 1 - a.tp && duel.is_chain_disablable(ev)
}

fn tecuhtlica_filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_MEMENTOAL_TECUHTLICA)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel) && chkc.is_code(duel, CARD_MEMENTOAL_TECUHTLICA));
    }
    if chk == 0 {
        return Ok(!duel.has_flag_effect(tp, ID) && duel.is_existing_target(tecuhtlica_filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(tp, tecuhtlica_filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.register_flag_effect(tp, ID, RESET_CHAIN, EFFECT_FLAG_OATH, 1);
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let reset = Some(RESET_EVENT | RESETS_STANDARD);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
        && tc.update_attack(duel, -1000, reset, Some(c), None) == -1000
        && tc.update_defense(duel, -1000, reset, Some(c), None) == -1000
    {
        duel.negate_effect(a.ev as u8);
    }
    Ok(())
}
