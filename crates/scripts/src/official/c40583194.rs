// Force Resonator
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 40583194;

// On the field beside a face-up monster (Kojikocy) to give the effect to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (1184620, MZONE), (1184620, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, Some(e.get_handler(duel).into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        e1.set_operation(duel, atkop);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_owner(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_target_range(duel, 0, 0xff);
    e1.set_value(
        duel,
        Value::EffectCard(|duel, _, _, c| c.is_monster(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_MZONE))),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, tp);
    Ok(())
}
