// Bri Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 52840598;

// On the field with Kojikocy (Level 4 Normal Monster) for Stardust Dragon in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (CARD_STARDUST_DRAGON, EXTRA),
    (CARD_STARDUST_DRAGON, EXTRA),
    (CARD_STARDUST_DRAGON, EXTRA),
];
// A continuous effect forms no chain link, so the check is the card used as Synchro material.
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, con);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn con(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(sync) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 600);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    sync.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DISABLE);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    sync.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DISABLE_EFFECT);
    e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
    sync.register_effect(duel, e3);
    Ok(())
}
