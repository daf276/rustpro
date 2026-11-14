// Abyss-scale of the Mizuchi
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::ConditionFn;
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 72932673;

pub const LISTED_SERIES: &[u16] = &[SET_MERMAIL];
// Mermail Abysspikes to equip, and the opponent's Dark Holes to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (58471134, MZONE),
    (58471134, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// The negation is a continuous effect forming no chain link.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_CHAIN_DISABLED)];

pub struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_set_card(duel, &[SET_MERMAIL])
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_abyss_scale(duel, c, 800, negcon);
}

// The equip procedure, ATK gain and negation the Abyss-scales share, which differ in the ATK and the negated effects.
pub(super) fn add_abyss_scale(duel: &mut Duel, c: CardId, atk: i32, negcon: ConditionFn) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, atk);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_CHAIN_SOLVING);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_condition(duel, negcon);
    e4.set_operation(duel, |duel, a| {
        if duel.negate_effect(a.ev as u8) {
            duel.send_to_grave(a.e.get_handler(duel), REASON_EFFECT)?;
        }
        Ok(())
    });
    c.register_effect(duel, e4);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.rp != a.tp && loc & LOCATION_SZONE != 0 && a.re.is_some_and(|re| re.is_spell_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
}
