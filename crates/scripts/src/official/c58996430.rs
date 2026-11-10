// Wulf, Lightsworn Beast
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 58996430;

// In the Deck to be sent by Lyla and Jain (Lightsworn monsters sending cards from the Deck at the end of the turn).
pub const SUPPORT: &[(u32, u16)] = &[(ID, DECK), (ID, DECK), (ID, DECK), (22624373, MZONE), (96235275, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_previous_location(duel) == LOCATION_DECK);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e2.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, se, _, _, _, _| se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS))),
    );
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
