// Ninjitsu Art of Decoy
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 89628781;

struct Procedure;

impl PersistentProcedure for Procedure {
    const P: Option<u8> = Some(0);

    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_faceup(duel) && c.is_set_card(duel, &[SET_NINJA])
    }
}

// In the hand to activate on Armed Ninja and Ninja Grandmaster Sasuke.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (9076207, MZONE), (4041838, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Persistent::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, Persistent::target_filter);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}
