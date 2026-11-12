// Daigusto Falcos
use super::c10755984::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 34109611;

// In the Extra Deck, Synchro Summoned with Gusto Squirro (a Level 2 Tuner) and Gusto Griffin (Level 2).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (336369, MZONE),
    (336369, MZONE),
    (42082363, MZONE),
    (42082363, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GUSTO])
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    for tc in g {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, 600);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
