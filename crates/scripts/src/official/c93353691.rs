// Driven Daredevil
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 93353691;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_WARRIOR, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_WARRIOR, Some(sc), sumtype, tp))
    }
}

// Two on the field, attacked by the opponent's strong monsters (Elfnote Lucina).
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, Value::Card(indes));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn indes(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    let ec = e.get_handler(duel);
    i32::from(Some(ec) == duel.get_attack_target() && c.get_attack(duel) > ec.get_attack(duel))
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let attacker = duel.get_attacker();
    // The Lua errors on a nil attacker.
    unported!(duel, a.ev == 1 && attacker.is_none());
    a.ev == 1 && attacker.is_some_and(|ac| ac.is_relate_to_battle(duel) && ac.get_attack(duel) > a.e.get_handler(duel).get_attack(duel))
}

fn destg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let attacker = duel.get_attacker();
    duel.set_operation_info(0, CATEGORY_DESTROY, attacker.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, _: &Args) -> Script<()> {
    let attacker = duel.get_attacker();
    // The Lua errors on a nil attacker.
    unported!(duel, attacker.is_none());
    if let Some(attacker) = attacker
        && attacker.is_relate_to_battle(duel)
    {
        duel.destroy(attacker, REASON_EFFECT)?;
    }
    Ok(())
}
