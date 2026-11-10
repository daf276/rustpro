// Revived King Ha Des
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

pub const LISTED_NAMES: &[u32] = &[33420078];
pub const MATERIAL: &[u32] = &[33420078];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[33420078])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_ZOMBIE, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with Plaguespreader Zombie and a Level 4 Zombie (Zombino, Temple of Skulls), with the
// opponent's monsters to battle. It has only a continuous effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (5309481, EXTRA),
    (5309481, EXTRA),
    (33420078, MZONE),
    (33420078, MZONE),
    (14575467, MZONE),
    (732302, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (Some(attacker), Some(defender)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return Ok(());
    };
    let c = a.e.get_handler(duel);
    let p = c.get_controler(duel);
    let destroyed = |c: CardId| c.is_status(duel, STATUS_BATTLE_DESTROYED);
    let tc = if attacker.get_controler(duel) == p && attacker.is_race(duel, RACE_ZOMBIE) && destroyed(defender) {
        defender
    } else if defender.get_controler(duel) == p && defender.is_race(duel, RACE_ZOMBIE) && destroyed(attacker) {
        attacker
    } else {
        return Ok(());
    };
    for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD & !(RESET_LEAVE | RESET_TOGRAVE));
        tc.register_effect(duel, e1);
    }
    Ok(())
}
