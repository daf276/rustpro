// Stygian Sergeants
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 86137485;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FIEND, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Copies on the field to attack the opponent's Sangan, and in the Extra Deck to Synchro Summon with Chain Resonator (Level 1
// Fiend Tuner) and Prevent Rat (Level 4).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (13764881, MZONE),
    (549481, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, atcon1);
    e1.set_operation(duel, atop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, atcon2);
    e2.set_operation(duel, atop2);
    c.register_effect(duel, e2);
}

fn is_monster_in_grave(duel: &Duel, c: CardId) -> bool {
    let bc = c.get_battle_target(duel);
    // The Lua errors on a nil battle target.
    unported!(duel, bc.is_none());
    bc.is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
}

fn atcon1(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_turn_player(a.tp)
        && is_monster_in_grave(duel, c)
        && duel.check_mut(|duel| c.can_chain_attack(duel))
        && c.is_status(duel, STATUS_OPPO_BATTLE)
}

fn boost(duel: &mut Duel, a: &Args) -> Script<bool> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(false);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 800);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
    c.register_effect(duel, e1);
    Ok(true)
}

fn atop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if boost(duel, a)? {
        duel.chain_attack();
    }
    Ok(())
}

fn atcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_turn_player(1 - a.tp) && c.is_relate_to_battle(duel) && c.is_status(duel, STATUS_OPPO_BATTLE) && is_monster_in_grave(duel, c)
}

fn atop2(duel: &mut Duel, a: &Args) -> Script<()> {
    boost(duel, a)?;
    Ok(())
}
