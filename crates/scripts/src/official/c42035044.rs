// Panther Warrior
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 42035044;

// On the field with Burglar to Tribute, to attack the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (6297941, MZONE), (6297941, MZONE), (26202165, MZONE | OPPONENT)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ATTACK_COST);
    e1.set_cost_condition(duel, |duel, e, _, tp, _| {
        duel.check_release_group_full(tp, aux::TRUE, 1, release_options(duel, e, tp))
    });
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
}

fn release_options(duel: &Duel, e: EffectId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        exception: Some(e.get_handler(duel)),
        ..ReleaseOptions::new(tp)
    }
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.is_attack_cost_paid() == 2 || !c.is_location(duel, LOCATION_MZONE) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let first = duel.is_attack_cost_paid() == 0;
    let g = duel.get_release_group(tp).filter(duel, aux::TRUE, Some(c.into()));
    if let Some(tc) = g.select_unselect(duel, Some(CardSet::default()), tp, first, first, 1, 1)? {
        duel.release(tc, REASON_COST)?;
        duel.attack_cost_paid();
    } else {
        duel.attack_cost_paid_full(2);
    }
    Ok(())
}
