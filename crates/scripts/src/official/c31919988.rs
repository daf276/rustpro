// Dark Diviner
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 31919988;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_INSECT, Some(sc), sumtype, tp))
    }
}

// Two on the field, with a weak (Psychic Kappa) and a strong (Elfnote Lucina) monster on the opponent's side to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_condition(duel, atcon);
    e2.set_target(duel, attg);
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DECKDES);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_condition(duel, ddcon);
    e3.set_target(duel, ddtg);
    e3.set_operation(duel, ddop);
    c.register_effect(duel, e3);
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_battle_target(duel)
        .is_some_and(|tc| tc.is_faceup(duel) && tc.get_attack(duel) > c.get_attack(duel))
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    // The Lua errors on a nil battle target.
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    unported!(duel, tc.is_none());
    if let Some(tc) = tc {
        tc.create_effect_relation(duel, a.e);
    }
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = c.get_battle_target(duel);
    // The Lua errors on the nil battle target once it passes the checks on this card.
    unported!(duel, tc.is_none() && c.is_relate_to_effect(duel, e) && c.is_faceup(duel));
    let Some(tc) = tc else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e)
        || c.is_facedown(duel)
        || tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
        || tc.get_attack(duel) <= c.get_attack(duel)
    {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, c.get_attack(duel));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
    tc.register_effect(duel, e1);
    Ok(())
}

fn ddcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE) && tc.is_reason(duel, REASON_BATTLE))
}

fn ddtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, 1 - a.tp, 3);
    Ok(false)
}

fn ddop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(1 - a.tp, 3, REASON_EFFECT)?;
    Ok(())
}
