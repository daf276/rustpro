// Sword Breaker
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 64689404;

// On the field with materials, with the opponent's strong Normal Monsters of five races (Black Luster Soldier, Tri-Horned Dragon, Gogiga Gagagigo,
// Spiral Serpent, Cosmo Queen) that attack it, so that the announced race is often one of them. The destruction needs the announced race to match
// the battled monster, which random play reached once in 64 scenarios.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (10000100, MZONE | OPPONENT),
    (39111158, MZONE | OPPONENT),
    (39674352, MZONE | OPPONENT),
    (32626733, MZONE | OPPONENT),
    (38999506, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, ractg);
    e1.set_operation(duel, racop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
}

fn ractg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let Some(e2) = e.get_label_object(duel) else { return Ok(false) };
    if chk == 0 {
        return Ok(e2.get_label(duel) != RACE_ALL as i64);
    }
    if c.get_flag_effect(duel, ID) == 0 {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
        e2.set_label(duel, 0);
    }
    let prc = e2.get_label(duel) as u64;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RACE);
    let rc = duel.announce_race(tp, 1, RACE_ALL - prc)?;
    e.set_label(duel, rc as i64);
    Ok(false)
}

fn racop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let Some(e2) = e.get_label_object(duel) else { return Ok(()) };
        let rc = e.get_label(duel) | e2.get_label(duel);
        e2.set_label(duel, rc);
        c.set_hint(duel, CHINT_RACE, rc as u64);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_flag_effect(duel, ID) != 0
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_faceup(duel) && bc.is_race(duel, a.e.get_label(duel) as u64))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
