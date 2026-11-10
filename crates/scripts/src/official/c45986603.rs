// Snatch Steal
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 45986603;

// In the hand to take a Sangan from the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl EquipProcedure for Procedure {
    const P: Option<u8> = Some(1);
    const EQLIMIT: Option<fn(&Duel, EffectId, CardId) -> i32> = Some(eqlimit);

    fn f(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
        aux::check_steal_equip(c, duel, e, tp)
    }

    fn tg(duel: &mut Duel, a: &Args, tc: Option<CardId>) -> Script<()> {
        a.e.set_category(duel, CATEGORY_CONTROL + CATEGORY_EQUIP);
        duel.set_operation_info(0, CATEGORY_CONTROL, tc.map(CardSet::from), 1, 0, 0);
        Ok(())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, rectg);
    e2.set_operation(duel, recop);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_EQUIP);
    e4.set_code(duel, EFFECT_SET_CONTROL);
    e4.set_value(duel, Value::Card(|duel, e, _| i32::from(e.get_handler_player(duel))));
    c.register_effect(duel, e4);
}

fn eqlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(e.get_handler_player(duel) != c.get_controler(duel) || e.get_handler(duel).get_equip_target(duel) == Some(c))
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, 1 - tp, 1000);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
