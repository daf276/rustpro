// Draco Berserker of the Tenyi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 5041348;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned with Tune Warrior and Peacock (a Level 5 Normal Monster).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (74093656, MZONE), (20624263, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, rmcon);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| a.rp == 1 - a.tp && re.is_monster_effect(duel) && re.get_handler(duel).is_relate_to_effect(duel, re))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(rc) = a.re.map(|re| re.get_handler(duel)) else {
        return Ok(false);
    };
    if chk == 0 {
        return Ok(rc.is_able_to_remove(duel) && !rc.is_location(duel, LOCATION_REMOVED));
    }
    duel.set_target_card(rc);
    duel.set_operation_info(
        0,
        CATEGORY_REMOVE,
        Some(rc.into()),
        1,
        rc.get_controler(duel),
        rc.get_location(duel) as i32,
    );
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(rc) = a.re.map(|re| re.get_handler(duel))
        && rc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(rc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none() && duel.get_attacker() == Some(c) && c.is_relate_to_battle(duel));
    duel.get_attacker() == Some(c)
        && c.is_relate_to_battle(duel)
        && bc.is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_type(duel, TYPE_EFFECT))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) {
        duel.set_target_card(bc);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || !c.is_relate_to_battle(duel) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, tc.get_base_attack(duel).max(0));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    let e2 = duel.create_effect(c);
    e2.set_description(duel, 3202);
    e2.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    Ok(())
}
