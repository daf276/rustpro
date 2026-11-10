// Gearfried the Iron Knight
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 423705;

// On the field, with Axe of Despair in the hand to equip it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (40619825, HAND), (40619825, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_EQUIP);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.get_equip_target(duel) == Some(ec)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let eg = a.eg.unwrap_or_default();
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(eg.is_exists(duel, |tc, duel| filter(tc, duel, c), 1, None));
    }
    let dg = eg.filter(duel, |tc, duel| filter(tc, duel, c), None);
    duel.set_target_card(dg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    duel.destroy(tg, REASON_EFFECT)?;
    Ok(())
}
