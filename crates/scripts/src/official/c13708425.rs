// Flare Resonator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 13708425;

// On the field as the Tuner of a Synchro Summon of Red Dragon Archfiend with Shapesnatch, a Level 5 Normal Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (4035199, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::SpecialSummonedCode(CARD_RED_DRAGON_ARCHFIEND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, |_, a| a.r == REASON_SYNCHRO);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(sync) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 300);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sync.register_effect(duel, e1);
    Ok(())
}
