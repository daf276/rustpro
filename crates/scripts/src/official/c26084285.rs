// Gravekeeper's Watcher
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 26084285;

// The opponent's Card Destruction and Hand Destruction to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (72892473, HAND | OPPONENT),
    (72892473, HAND | OPPONENT),
    (74519184, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.ep == a.tp || (!re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && !re.is_monster_effect(duel)) || !duel.is_chain_negatable(a.ev as u8)
    {
        return false;
    }
    duel.get_operation_info(a.ev as u8, CATEGORY_HANDES)
        .is_some_and(|info| info.op_player == a.ep || info.op_player == PLAYER_ALL)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
