// Anti-Spell Fragrance
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 58921041;

// One in the hand to activate and one Set, against the opponent's Raigeki and Dark Hole in hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (12580477, HAND | OPPONENT), (53129443, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SSET);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_operation(duel, aclimset);
    c.register_effect(duel, e3);
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    if !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) || !re.is_spell_effect(duel) {
        return false;
    }
    let c = re.get_handler(duel);
    !c.is_location(duel, LOCATION_SZONE) || c.get_flag_effect(duel, ID) > 0
}

fn aclimset(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN, 0, 1);
    }
    Ok(())
}
