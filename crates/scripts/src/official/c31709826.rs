// Revival Jam
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 31709826;

// Copies on the field to be battle destroyed by the opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
    });
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_operation(duel, |duel, a| {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
    });
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    c.reset_flag_effect(duel, ID);
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP_DEFENSE)?;
    Ok(())
}
