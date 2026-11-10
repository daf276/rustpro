// Heat Wave
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 45141013;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1) && !duel.check_phase_activity());
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_target_range(duel, 1, 1);
    e1.set_target_spsummon(duel, sumlimit);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, a.tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    duel.register_effect(e2, a.tp);
    Ok(())
}

fn sumlimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    if c.is_monster(duel) {
        c.is_type(duel, TYPE_EFFECT)
    } else {
        c.is_original_type(duel, TYPE_EFFECT)
    }
}
