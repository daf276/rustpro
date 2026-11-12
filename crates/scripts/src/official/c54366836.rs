// Number 54: Lion Heart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 54366836;
pub const XYZ_NUMBER: u32 = 54;

// In the Extra Deck, Xyz Summoned with three Level 1 monsters, with the opponent's Elfnote Lucinas to attack it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (3557275, MZONE),
    (59983499, MZONE),
    (69380702, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_attack_pos(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.ep == a.tp && a.e.get_handler(duel).is_relate_to_battle(duel));
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_battle_target(duel)
            .is_some_and(|bc| bc.is_controler(duel, 1 - a.tp))
    });
    // The Lua passes the Cost.SoftOncePerBattle factory itself instead of calling it, so it never limits the effect.
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_operation(duel, reversedamop);
    c.register_effect(duel, e3);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(a.ev);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, a.ev as i32);
    Ok(false)
}

fn reversedamop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE_CAL);
    duel.register_effect(e1, tp);
    Ok(())
}
