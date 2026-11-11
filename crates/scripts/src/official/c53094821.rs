// Toon Terror
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53094821;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// Set with Toon World and a Toon Mermaid on the field, to negate the opponent's Raigeki and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (CARD_TOON_WORLD, SZONE),
    (CARD_TOON_WORLD, SZONE),
    (CARD_TOON_WORLD, SZONE),
    (65458948, MZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_TOON_WORLD)
}

fn cfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_TOON)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    if !duel.is_existing_matching_card(cfilter, tp, LOCATION_ONFIELD, 0, 1, None)
        || !duel.is_existing_matching_card(cfilter2, tp, LOCATION_MZONE, 0, 1, None)
        || !duel.is_chain_negatable(a.ev as u8)
    {
        return false;
    }
    a.re.is_some_and(|re| re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
