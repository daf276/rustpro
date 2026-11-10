// Mystical Beast of Serket
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 89194033;

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS];
// On the field with Temple of the Kings in the hand to activate, and Sangan on the opponent's side to destroy in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (CARD_TEMPLE_OF_THE_KINGS, HAND),
    (CARD_TEMPLE_OF_THE_KINGS, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_SELF_DESTROY);
    e1.set_condition(duel, descon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_BATTLE_DESTROY_REDIRECT);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(desfilter, a.e.get_handler(duel).get_controler(duel), LOCATION_SZONE, 0, 1, None)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel) && c.is_relate_to_battle(duel)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 500);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}
