// White Wing Magician
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 11067666;

// In the Pendulum Zone, with Stargazer Magician and Dark Magician on the field for the opponent's Mage Power and Mind Control to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (94415058, MZONE),
    (36996508, MZONE),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
// The activation is a continuous effect that forms no chain link.
pub const CHECKS: &[Check] = &[Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    e2.set_condition(duel, rmcon);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
        && c.is_race(duel, RACE_SPELLCASTER)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_controler(duel, tp)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    a.e.get_handler(duel).get_flag_effect(duel, ID) == 0
        && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)
        && g.is_some_and(|g| g.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
        && duel.is_chain_disablable(a.ev as u8)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if duel.select_effect_yes_no(a.tp, c)? {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        if duel.negate_effect(a.ev as u8) {
            duel.break_effect()?;
            duel.destroy(c, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_pendulum_summoned(duel) && c.is_reason(duel, REASON_MATERIAL) && c.is_reason(duel, REASON_SYNCHRO)
}
