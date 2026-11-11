// Naturia Butterfly
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 42110434;

// On the field, and the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        a.eg.and_then(|eg| eg.get_first()).is_some_and(|c| c.get_controler(duel) != a.tp)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(a.tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 1, REASON_EFFECT)?;
    duel.negate_attack()?;
    Ok(())
}
