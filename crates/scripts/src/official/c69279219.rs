// My Body as a Shield
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

// Raigeki and Dark Hole in the opponent's hand to negate, with a monster of the player's own for them to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (26202165, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::pay_lp::<1500>);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_on_field(duel) && c.is_monster(duel)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.tp == a.ep || !duel.is_chain_negatable(a.ev as u8) {
        return false;
    }
    if !re.is_monster_effect(duel) && !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return false;
    }
    duel.get_operation_info(a.ev as u8, CATEGORY_DESTROY).is_some_and(|info| {
        info.op_cards
            .is_some_and(|tg| i32::from(info.op_count) + tg.filter_count(duel, cfilter, None) as i32 - tg.len() as i32 > 0)
    })
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
