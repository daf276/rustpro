// Grisaille Prison
use crate::prelude::*;
use crate::testing::{HAND, MZONE, SZONE};

const ID: u32 = 22888900;

// Set, with Summoned Skull in the hand to Tribute Summon from Gladiator Beast Andal on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (70781052, HAND),
    (70781052, HAND),
    (70781052, HAND),
    (90582719, MZONE),
    (90582719, MZONE),
    (90582719, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_tribute_summoned(duel) || c.is_ritual_summoned(duel) || c.is_fusion_summoned(duel))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let count = if duel.is_turn_player(1 - tp) { 2 } else { 1 };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_SYNCHRO + TYPE_XYZ));
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN, count);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_target_range(duel, 1, 1);
    e3.set_target_spsummon(duel, |_, _, _, _, sumtp, _, _, _| {
        sumtp & SUMMON_TYPE_SYNCHRO == SUMMON_TYPE_SYNCHRO || sumtp & SUMMON_TYPE_XYZ == SUMMON_TYPE_XYZ
    });
    e3.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN, count);
    duel.register_effect(e3, tp);
    Ok(())
}
