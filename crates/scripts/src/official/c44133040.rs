// Evil Eye of Selene
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};
use engine::constants::MSG_EQUIP;

const ID: u32 = 44133040;

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_set_card(duel, &[SET_EVIL_EYE])
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand to equip, with Catoblepas on the field to equip to and activate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (45955628, MZONE), (45955628, MZONE)];
// Only another Evil Eye Spell/Trap in the Graveyard pays for the Set effect, and none besides Selene itself is ported.
pub const CHECKS: &[Check] = &[
    Check::Message(MSG_EQUIP),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e3);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_ATKCHANGE);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e5.set_code(duel, EVENT_CHAINING);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_condition(duel, atkupcond);
    e5.set_operation(duel, atkupop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 1));
    e6.set_category(duel, CATEGORY_SET);
    e6.set_type(duel, EFFECT_TYPE_IGNITION);
    e6.set_range(duel, LOCATION_GRAVE);
    e6.set_count_limit(duel, 1, ID);
    e6.set_cost(duel, setcost);
    e6.set_target(duel, settg);
    e6.set_operation(duel, setop);
    c.register_effect(duel, e6);
}

fn atkupcond(duel: &Duel, a: &Args) -> bool {
    let (c, Some(re)) = (a.e.get_handler(duel), a.re) else {
        return false;
    };
    let rc = re.get_handler(duel);
    ((re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_spell_trap_effect(duel) && rc.is_set_card(duel, &[SET_EVIL_EYE]) && rc != c)
        || Some(rc) == c.get_equip_target(duel))
        && a.tp == a.rp
}

fn atkupop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let Some(ec) = c.get_equip_target(duel) else {
        unported!(duel, true);
        return Ok(());
    };
    if ec.update_attack(duel, 500, None, Some(c), None) == 500 && c.is_relate_to_effect(duel, a.e) {
        let lp = duel.get_lp(tp) - 500;
        duel.set_lp(tp, lp);
    }
    Ok(())
}

fn rfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_set_card(duel, &[SET_EVIL_EYE]) && c.is_able_to_remove_as_cost(duel) && !c.is_code(duel, ID)
}

fn setcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_lp_cost(tp, 1000) && duel.is_existing_matching_card(rfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    duel.pay_lp_cost(tp, 1000)?;
    Ok(false)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
