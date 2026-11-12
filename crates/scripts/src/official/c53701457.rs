// Number 28: Titanic Moth
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 53701457;
pub const XYZ_NUMBER: u32 = 28;

// On the field with Cyber Falcon attached by Xyz Import, to attack directly, or in the Extra Deck, Xyz Summoned with two Level 7 Normal
// Monsters (Gaia The Fierce Knight), with Cyber Falcon to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (6368038, MZONE),
    (6368038, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (14602126, HAND),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_MZONE, 0) <= 1
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e2.set_condition(duel, rdcon);
    e2.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, HALF_DAMAGE>));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_condition(duel, |_, a| a.ep != a.tp);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
}

fn rdcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let tp = a.e.get_handler_player(duel);
    duel.get_attack_target().is_none()
        && c.get_effect_count(duel, EFFECT_DIRECT_ATTACK) < 2
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_field_group_count(tp, 0, LOCATION_HAND);
    if chk == 0 {
        return Ok(ct > 0);
    }
    duel.set_target_player(1 - tp);
    let dam = ct as u32 * 500;
    duel.set_target_param(dam);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = duel.get_field_group_count(p, LOCATION_HAND, 0) as u32 * 500;
    duel.damage(p, dam, REASON_EFFECT)?;
    Ok(())
}
