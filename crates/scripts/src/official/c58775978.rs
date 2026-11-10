// Nightmare's Steelcage
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 58775978;
const PYRO_CLOCK: u32 = 1082946;

// In the hand, with Pyro Clock of Destiny to advance its destruction.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (PYRO_CLOCK, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_REMAIN_FIELD);
    c.register_effect(duel, e3);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE));
    }
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.set_turn_counter(duel, 0);
    let reset = RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_operation(duel, desop);
    e1.set_reset_full(duel, reset, 2);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(
        duel,
        EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE,
    );
    e3.set_code(duel, PYRO_CLOCK);
    e3.set_label_object(duel, Some(e1));
    e3.set_owner_player(duel, tp);
    e3.set_operation(duel, resetop);
    e3.set_reset_full(duel, reset, 2);
    c.register_effect(duel, e3);
    Ok(false)
}

fn resetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(e1) = a.e.get_label_object(duel) else { return Ok(()) };
    desop(
        duel,
        &Args {
            e: e1,
            re: Some(a.e),
            ..*a
        },
    )
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = c.get_turn_counter(duel) + 1;
    c.set_turn_counter(duel, ct);
    if ct == 2 {
        duel.destroy(c, REASON_RULE)?;
        if let Some(re) = a.re {
            re.reset(duel);
        }
    }
    Ok(())
}
