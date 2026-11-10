// Jain, Lightsworn Paladin
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 96235275;

// On the field to send cards from the Deck at the end of the turn, with an opponent's monster to attack into.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (12580477, DECK), (12580477, DECK), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_condition(duel, |duel, a| {
        let ph = duel.get_current_phase();
        (ph == PHASE_DAMAGE || ph == PHASE_DAMAGE_CAL)
            && duel.get_attacker() == Some(a.e.get_handler(duel))
            && duel.get_attack_target().is_some()
    });
    e1.set_value(duel, 300);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_code(duel, EVENT_PHASE | PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 2);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 2, REASON_EFFECT)?;
    Ok(())
}
