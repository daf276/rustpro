// Performapal Five-Rainbow Magician
use crate::prelude::*;
use crate::testing::{Check, GRAVE, PZONE};

const ID: u32 = 19619755;

// In the Graveyard to return to a Pendulum Zone when its controller Sets a card, and in a Pendulum Zone.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (ID, PZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SSET);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_controler(duel, a.tp), 1, None)
    });
    e1.set_target(duel, pentg);
    e1.set_operation(duel, penop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, sumtp, _, _, _| {
        sumtp & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM && !c.is_location(duel, LOCATION_EXTRA)
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, _, c| {
        facedown_count(duel, c.get_controler(duel), LOCATION_SZONE, 0) == 0
    });
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_range(duel, LOCATION_PZONE);
    e4.set_target_range(duel, 1, 1);
    e4.set_value(duel, Value::EffectPlayer(limval));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e5.set_range(duel, LOCATION_PZONE);
    e5.set_target_range(duel, LOCATION_MZONE, 0);
    e5.set_condition(duel, |duel, a| {
        facedown_count(duel, a.e.get_handler_player(duel), LOCATION_SZONE, 0) >= 4
    });
    e5.set_value(duel, Value::Card(|duel, _, c| c.get_base_attack(duel) * 2));
    c.register_effect(duel, e5);
    let e6 = e5.clone_effect(duel);
    e6.set_target_range(duel, 0, LOCATION_MZONE);
    e6.set_condition(duel, |duel, a| {
        facedown_count(duel, a.e.get_handler_player(duel), 0, LOCATION_SZONE) >= 4
    });
    c.register_effect(duel, e6);
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_pendulum_zones(a.tp));
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.check_pendulum_zones(a.tp) {
        duel.move_to_field(c, a.tp, a.tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn countfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.get_sequence(duel) < 5
}

fn facedown_count(duel: &Duel, player: u8, location1: u32, location2: u32) -> usize {
    duel.get_matching_group_count(countfilter, player, location1, location2, None)
}

fn limval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    let tp = rc.get_controler(duel);
    rc.is_location(duel, LOCATION_MZONE) && re.is_monster_effect(duel) && facedown_count(duel, tp, LOCATION_SZONE, 0) == 0
}
