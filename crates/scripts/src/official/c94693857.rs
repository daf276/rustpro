// Deskbot 001
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 94693857;

// The trigger needs two Machines Special Summoned at once, which no ported card does.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0)))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn machine(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE)
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(machine, c.get_controler(duel), LOCATION_MZONE, 0, None) as i32 * 500
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, machine, 2, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_relate_to_effect(duel, e)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}
