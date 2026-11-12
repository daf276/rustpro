// Ragnaraika Chain Coils
use super::c42307760::{selfspop, selfsptg};
use super::c70514456::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 6908161;

// Link Summoned from Insect and Plant Normal Monsters, with a copy in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (58192742, MZONE),
    (58192742, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure<4>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_monster_effect(duel)) && a.rp == 1 - a.tp
    });
    e1.set_operation(duel, cnntactop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop::<ID, 3>);
    c.register_effect(duel, e2);
}

fn cnntactop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_location(duel, LOCATION_HAND) && re.is_monster_effect(duel)
}
