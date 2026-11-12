// The Fabled Unicore
use super::c54048462::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_CHAIN_NEGATED;

const ID: u32 = 44155002;

// In the Extra Deck, Synchro Summoned with The Fabled Chawa (a Level 1 Tuner) and Swordsman of Landstar (Level 3), against the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (29905795, MZONE),
    (29905795, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_CHAIN_NEGATED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.ep == tp || duel.get_field_group_count(tp, LOCATION_HAND, 0) != duel.get_field_group_count(tp, 0, LOCATION_HAND) {
        return Ok(());
    }
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_effect(a.ev as u8) && rc.is_relate_to_effect(duel, re) {
        duel.destroy(rc, REASON_EFFECT)?;
    }
    Ok(())
}
