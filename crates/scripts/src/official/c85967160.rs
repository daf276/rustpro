// Aromage Bergamot
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 85967160;

// Aromalilith Rosalina and Aromaseraphy Angelica in the hand to gain LP by discarding, and a second copy on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (24689197, HAND), (24689197, HAND), (16759958, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_PIERCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_condition(duel, pccon);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_PLANT));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_RECOVER);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |_, a| a.ep == a.tp);
    e2.set_target(duel, adtg);
    e2.set_operation(duel, adop);
    c.register_effect(duel, e2);
}

fn pccon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_lp(tp) > duel.get_lp(1 - tp)
}

fn adtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.ep == a.tp && a.e.get_handler(duel).is_relate_to_effect(duel, a.e))
}

fn adop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END | RESET_OPPO_TURN);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        c.register_effect(duel, e2);
    }
    Ok(())
}
