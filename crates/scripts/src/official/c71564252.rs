// Thunder King Rai-Oh
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 71564252;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_DECK, LOCATION_DECK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_SPSUMMON);
    e2.set_condition(duel, |duel, a| {
        a.tp != a.ep && a.eg.is_some_and(|eg| eg.len() == 1) && duel.get_current_chain() == 0
    });
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}
