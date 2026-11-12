// Noble Arms of Destiny
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 7452945;

// In the hand to equip to Noble Knight Artorigus, then destroyed by the opponent's Lightning Storm to equip again.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (92125819, MZONE),
    (14532163, HAND | OPPONENT),
    (14532163, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_WARRIOR)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & (REASON_BATTLE + REASON_EFFECT) != 0));
    e2.set_count_limit(duel, 1, 0);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_EQUIP);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, eqcon);
    e4.set_target(duel, eqtg);
    e4.set_operation(duel, eqop);
    c.register_effect(duel, e4);
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && c.check_unique_on_field(duel, tp) {
        duel.equip(tp, c, tc)?;
    }
    Ok(())
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_reason(duel, REASON_DESTROY)
        && c.check_unique_on_field(duel, a.tp)
}

fn eqfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_NOBLE_KNIGHT]) && c.is_race(duel, RACE_WARRIOR)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && eqfilter2(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_relate_to_effect(duel, e)
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(eqfilter2, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter2, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}
