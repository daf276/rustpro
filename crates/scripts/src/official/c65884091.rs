// Evilswarm Thanatos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 65884091;

// On the field with Xyz Import to attach Sangan, and the opponent's Sangan to activate effects into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3101);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, Value::Effect(efilter));
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_active_type(duel, TYPE_EFFECT) && te.get_owner(duel) != e.get_owner(duel)
}
