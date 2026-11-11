// Heavy Mech Support Armor
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 39890958;

struct Procedure;

impl UnionProcedure for Procedure {
    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_MACHINE)
    }
}

// On the field with a Machine monster to equip: Mechanicalchaser.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (7359741, MZONE)];
// Destruction substitutions are too rare to assert.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Report(&Check::Replaced)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_target(duel, sumtg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_UNION) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None) && duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
