// R-Genex Oracle
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 10178757;

// No ported Genex card sends a card from the Deck to the hand by an effect, so the trigger is not reachable.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, DECK)];
pub const CHECKS: &[Check] = &[Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_set_card(duel, &[SET_GENEX]))));
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.r & REASON_EFFECT > 0
        && a.re.is_some_and(|re| re.get_handler(duel).is_set_card(duel, &[SET_GENEX]))
        && c.get_previous_location(duel) == LOCATION_DECK
        && c.is_previous_controler(duel, a.tp)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    Ok(())
}
