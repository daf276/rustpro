// Dark Magic Attack
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 2314238;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];

// In the hand with Dark Magician on the field, and Spells and Traps of the opponent to destroy: Raigeki and Dark Hole copies.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, MZONE),
    (12580477, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_DARK_MAGICIAN));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_spell_trap, a.tp, 0, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let sg = duel.get_matching_group(CardId::is_spell_trap, a.tp, 0, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let sg = duel.get_matching_group(CardId::is_spell_trap, a.tp, 0, LOCATION_ONFIELD, Some(c.into()));
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
