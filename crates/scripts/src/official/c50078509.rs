// Fiendish Chain
use engine::constants::MSG_CARD_TARGET;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

struct Procedure;

impl PersistentProcedure for Procedure {
    const CATEGORY: u64 = CATEGORY_DISABLE;
    const HINT2: Option<u32> = Some(TIMINGS_CHECK_MONSTER);

    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
    }

    fn tg(duel: &mut Duel, _: &Args, tc: CardId) -> Script<()> {
        duel.set_operation_info(0, CATEGORY_DISABLE, Some(tc.into()), 1, 0, 0);
        Ok(())
    }
}

// Set, to be activated on the opponent's Effect Monster: Elfnote Fortuna.
pub const SUPPORT: &[(u32, u16)] = &[(50078509, SZONE), (85976588, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CARD_TARGET)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Persistent::add_procedure::<Procedure>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, Persistent::target_filter);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_CANNOT_ATTACK);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_code(duel, EVENT_LEAVE_FIELD);
    e5.set_condition(duel, descon);
    e5.set_operation(duel, |duel, a| duel.destroy(a.e.get_handler(duel), REASON_EFFECT).map(|_| ()));
    c.register_effect(duel, e5);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if c.is_status(duel, STATUS_DESTROY_CONFIRMED) {
        return false;
    }
    c.get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.is_contains(tc)) && tc.is_reason(duel, REASON_DESTROY))
}
