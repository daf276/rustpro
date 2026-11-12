// Constellar Praesepe
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 2091298;

// On the field with Xyz Import to attach Sangan, Constellar Castor to Xyz Summon it, and the opponent's attackers to fight in the Damage Step.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (33302589, MZONE),
    (33302589, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_CONSTELLAR], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition_mut(duel, condition);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &mut Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    if duel.get_current_phase() != PHASE_DAMAGE || duel.is_damage_calculated() {
        return false;
    }
    let attacker = duel.get_attacker();
    // Lua errors on a nil attacker
    unported!(duel, attacker.is_none());
    let Some(attacker) = attacker else { return false };
    let tc = if attacker.is_controler(duel, 1 - tp) {
        duel.get_attack_target()
    } else {
        Some(attacker)
    };
    e.set_label_object_group(duel, tc.map(Into::into));
    tc.is_some_and(|tc| tc.is_faceup(duel) && tc.is_set_card(duel, &[SET_CONSTELLAR]) && tc.is_relate_to_battle(duel))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    // Lua errors on a nil label object
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_relate_to_battle(duel) || tc.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, 1000);
    tc.register_effect(duel, e1);
    Ok(())
}
