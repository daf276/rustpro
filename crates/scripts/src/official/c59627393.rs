// Number 105: Battlin' Boxer Star Cestus
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 59627393;
pub const XYZ_NUMBER: u32 = 105;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

// Sangan on the opponent's side to attack into.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_ATTACK | TIMING_BATTLE_PHASE, TIMING_ATTACK | TIMING_BATTLE_PHASE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let (bc1, bc2) = duel.get_battle_monster(a.tp);
    bc2.is_some() && bc1.is_some_and(|bc1| bc1.is_set_card(duel, &[SET_BATTLIN_BOXER]) && bc1.is_faceup(duel))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let (bc1, bc2) = duel.get_battle_monster(a.tp);
    unported!(duel, bc1.is_none() || bc2.is_none());
    let (Some(bc1), Some(bc2)) = (bc1, bc2) else { return Ok(()) };
    if bc2.is_relate_to_battle(duel) && bc2.is_negatable_monster(duel) {
        bc2.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
    }
    if bc1.is_relate_to_battle(duel) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3000);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        bc1.register_effect_full(duel, e1, true);
        let e2 = e1.clone_effect(duel);
        e2.set_description(duel, 3112);
        e2.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
        bc1.register_effect_full(duel, e2, true);
    }
    Ok(())
}
