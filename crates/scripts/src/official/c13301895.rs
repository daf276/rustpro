// Fallen Paradise
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 13301895;

pub const LISTED_NAMES: &[u32] = &[6007213, 32491822, 69890967, 43378048];
// In the hand to activate, with Uria and Hamon on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (6007213, MZONE), (32491822, MZONE)];
// Counts both the activation and the draw, neither of which has a description.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| is_listed(c, duel));
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp == 1 - e.get_handler_player(duel)));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_DRAW);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(drcfilter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e4.set_target(duel, stock::draw_target::<2>);
    e4.set_operation(duel, drop);
    c.register_effect(duel, e4);
}

fn is_listed(c: CardId, duel: &Duel) -> bool {
    LISTED_NAMES.iter().any(|&code| c.is_code(duel, code))
}

fn drcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && is_listed(c, duel)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    stock::draw(duel, a)
}
