// Magical Musket - Last Stand
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 29628180;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];
// In the hand to activate with Magical Musketeer Kidbrave on the field, against the opponent's Dark Hole and Pot of Greed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (5230799, MZONE),
    (5230799, MZONE),
    (53129443, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let musket = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    a.rp != a.tp
        && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(musket, a.tp, LOCATION_MZONE, 0, 1, None)
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
