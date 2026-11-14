// Magic Deflector
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e1.set_target_card(duel, distg);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_operation(duel, disop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, a.tp);
    Ok(())
}

fn is_deflected(tpe: u32) -> bool {
    tpe & TYPE_SPELL != 0 && tpe & (TYPE_EQUIP + TYPE_FIELD + TYPE_CONTINUOUS + TYPE_QUICKPLAY) != 0
}

fn distg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    is_deflected(c.get_type(duel))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let tl: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    if tl & LOCATION_SZONE != 0 && is_deflected(re.get_active_type(duel)) {
        duel.negate_effect(a.ev as u8);
    }
    Ok(())
}
