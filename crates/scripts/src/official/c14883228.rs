// Typhoon
use crate::prelude::*;

const ID: u32 = 14883228;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_condition(duel, handcon);
    c.register_effect(duel, e2);
}

fn handcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    !duel.is_existing_matching_card(CardId::is_spell_trap, tp, LOCATION_ONFIELD, 0, 1, None)
        && duel.is_existing_matching_card(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 2, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
