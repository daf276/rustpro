// Steelswarm Roach
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 37926346;

// On the field with Xyz Import to attach Sangan, and the opponent's Monster Reborn on a Level 6 Normal Monster (Frostosaurus) to
// Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (83764718, HAND | OPPONENT),
    (6631034, GRAVE | OPPONENT),
    (6631034, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_SPSUMMON);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_current_chain() == 0 && a.eg.is_some_and(|eg| eg.is_exists(duel, filter, 1, None))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = a.eg.unwrap_or_default().filter(duel, filter, None);
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.eg.unwrap_or_default().filter(duel, filter, None);
    duel.negate_summon(g);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
