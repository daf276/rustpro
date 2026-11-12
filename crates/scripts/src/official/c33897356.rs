// Shootingcode Talker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 33897356;

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters, attacking Sangans of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (47226949, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect_label(duel, ID).is_some());
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_condition(duel, aux::bdcon);
    e3.set_operation(duel, bdop);
    c.register_effect(duel, e3);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.e.get_handler(duel).get_linked_group_count(duel) > 0)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let ct = c.get_linked_group_count(duel);
    if ct != 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
        e1.set_value(duel, ct as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        c.register_effect(duel, e1);
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_condition(duel, atkcon);
    e2.set_value(duel, -400);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e2);
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attack_target().is_some()
        && duel.is_phase(PHASE_DAMAGE_CAL)
        && duel.get_field_group_count(a.e.get_handler_player(duel), 0, LOCATION_MZONE) == 1
}

fn bdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(label) = c.get_flag_effect_label(duel, ID) {
        c.set_flag_effect_label(duel, ID, label as u32 + 1);
    } else {
        c.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, 1, 0);
    }
    Ok(())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = a.e.get_handler(duel).get_flag_effect_label(duel, ID).unwrap_or(0) as u32;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, ct as usize));
    }
    duel.set_target_player(tp);
    duel.set_target_param(ct);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}
