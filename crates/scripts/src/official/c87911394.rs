// Number C39: Utopia Ray Victory
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 87911394;

pub const LISTED_SERIES: &[u16] = &[SET_UTOPIA];
pub const XYZ_NUMBER: u32 = 39;

// In the Extra Deck, Xyz Summoned with Level 5 Normal Monsters, with the opponent's Elfnote Lucina and Ash Blossom to attack. The
// destroy effect needs a Utopia Xyz Material, which nothing places.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (2971090, MZONE),
    (4035199, MZONE),
    (4179849, MZONE),
    (13597785, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Message(MSG_ATTACK),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    e1.set_condition(duel, |duel, a| duel.get_attacker() == Some(a.e.get_handler(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_condition(duel, atkcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attack_target().is_some_and(|at| at.is_faceup(duel))
        && a.e
            .get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_UTOPIA]), 1, None)
}

fn atktg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(at) = duel.get_attack_target() {
        duel.set_target_card(at);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
        duel.adjust_instantly(Some(tc));
        let atk = tc.get_attack(duel);
        if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_code(duel, EFFECT_UPDATE_ATTACK);
            e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e3.set_value(duel, atk);
            e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
            c.register_effect(duel, e3);
        }
    }
    Ok(())
}
