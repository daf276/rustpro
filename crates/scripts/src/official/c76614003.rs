// Dark Tinker
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 76614003;

// On the field with Dark Hole in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_previous_controler(duel, a.tp)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) == 0 {
        return Ok(());
    }
    let g = duel.get_decktop_group(tp, 1);
    duel.confirm_cards(tp, g)?;
    let opt = duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?;
    if opt == 1
        && let Some(tc) = g.get_first()
    {
        duel.move_sequence(tc, 1)?;
    }
    Ok(())
}
