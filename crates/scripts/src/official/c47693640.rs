// Zombie Tiger
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 47693640;
const DECAYED_COMMANDER: u32 = 10209545;

struct Procedure;

impl UnionProcedure for Procedure {
    const OLD_EQUIP: bool = true;

    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, DECAYED_COMMANDER)
    }
}

pub const LISTED_NAMES: &[u32] = &[DECAYED_COMMANDER];
// On the field with Decayed Commander to equip, which then destroys the opponent's Sangan in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (DECAYED_COMMANDER, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// Destruction substitutions are too rare to assert, as is the equipped Decayed Commander winning a battle.
pub const CHECKS: &[Check] = &[
    Check::Resolves,
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
    Check::Report(&Check::Replaced),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    for (code, value) in [(EFFECT_UPDATE_ATTACK, 500), (EFFECT_UPDATE_DEFENSE, 500)] {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_EQUIP);
        e.set_code(duel, code);
        e.set_value(duel, value);
        e.set_condition(duel, Union::is_union_state);
        c.register_effect(duel, e);
    }
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_HANDES);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_code(duel, EVENT_BATTLE_DESTROYING);
    e5.set_condition(duel, hdcon);
    e5.set_target(duel, super::c10209545::hdtg);
    e5.set_operation(duel, super::c10209545::hdop);
    c.register_effect(duel, e5);
}

fn hdcon(duel: &Duel, a: &Args) -> bool {
    Union::is_union_state(duel, a)
        && a.eg
            .and_then(|eg| eg.get_first())
            .is_some_and(|tc| Some(tc) == a.e.get_handler(duel).get_equip_target(duel))
}
