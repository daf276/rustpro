// Assault Counter
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 76407432;

pub const LISTED_SERIES: &[u16] = &[SET_ASSAULT_MODE];
// Set on the field with an Assault Mode monster (Crimson Blader/Assault Mode), against the opponent's Sangan and Mage Power.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (50604072, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ASSAULT_MODE])
}

fn condition(duel: &Duel, a: &Args) -> bool {
    if !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None) || !duel.is_chain_negatable(a.ev as u8) {
        return false;
    }
    a.re.is_some_and(|re| re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
