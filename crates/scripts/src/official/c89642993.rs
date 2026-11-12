// Number 63: Shamoji Soldier
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 89642993;
pub const XYZ_NUMBER: u32 = 63;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, eftg);
    e1.set_operation(duel, efop);
    c.register_effect(duel, e1);
}

fn eftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let op = duel.select_option(a.tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?;
    a.e.set_label(duel, i64::from(op));
    Ok(false)
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.e.get_label(duel) == 0 {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE_START | PHASE_STANDBY);
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, drcon);
        e1.set_operation(duel, drop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
        duel.register_effect(e1, tp);
    } else {
        duel.recover(tp, 1000, REASON_EFFECT)?;
        duel.recover(1 - tp, 1000, REASON_EFFECT)?;
    }
    Ok(())
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.draw(tp, 1, REASON_EFFECT)?;
    duel.draw(1 - tp, 1, REASON_EFFECT)?;
    Ok(())
}
