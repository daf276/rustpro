// Chiron the Mage
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 16956455;

// On the field with Pot of Greed in hand to discard and Swords of Revealing Light and Mirror Wall of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (55144522, HAND),
    (55144522, HAND),
    (72302403, SZONE | OPPONENT),
    (22359980, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_discardable(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let handler = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(handler.into())));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
