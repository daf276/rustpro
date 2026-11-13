// World Legacy Struggle
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 58374719;

// Two Transcendosaurus Exaraptors on the field to point at each other, with Spells and Traps to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (56818742, MZONE),
    (56818742, MZONE),
    (4206964, SZONE),
    (4206964, SZONE),
    (4206964, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.get_mutual_linked_group_count(duel) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let sg = duel.select_target(
        tp,
        CardId::is_spell_trap,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        ct as u8,
        Some(c.into()),
    )?;
    let count = sg.map_or(0, |sg| sg.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, sg, count, 0, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
