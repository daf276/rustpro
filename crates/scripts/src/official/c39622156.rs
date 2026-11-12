// Number 26: Spaceway Octobypass
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 39622156;
pub const XYZ_NUMBER: u32 = 26;

// In the Extra Deck, Xyz Summoned with Level 3 Normal Monsters, and the opponent's monsters to attack and be attacked by.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (32864, MZONE),
    (2468169, MZONE),
    (7670542, MZONE),
    (8944575, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, |duel, a, chk, _| Cost::detach_from_self::<1, 1>(duel, a, chk));
    e1.set_operation(duel, btop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_DAMAGE_STEP_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e3.set_operation(duel, ctop);
    c.register_effect(duel, e3);
}

fn btop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let p = duel.get_turn_player();
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_condition(duel, atkcon);
    e1.set_target_card(duel, |duel, e, c| i64::from(c.get_field_id(duel)) != e.get_label(duel));
    e1.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
    duel.register_effect(e1, p);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_operation(duel, checkop);
    e2.set_label_object(duel, Some(e1));
    e2.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
    duel.register_effect(e2, p);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DIRECT_ATTACK);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_monster(duel));
    e3.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
    duel.register_effect(e3, p);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_value(duel, 1);
    e4.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
    duel.register_effect(e4, p);
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.get_flag_effect(a.e.get_handler_player(duel), ID) != 0
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if e.get_handler(duel).get_flag_effect(duel, ID) != 0 {
        return Ok(());
    }
    let first = a.eg.and_then(|eg| eg.get_first());
    unported!(duel, first.is_none());
    let Some(first) = first else { return Ok(()) };
    let fid = first.get_field_id(duel);
    duel.register_flag_effect(e.get_handler_player(duel), ID, RESETS_STANDARD_PHASE_END, 0, 1);
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, i64::from(fid));
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.get_attack_target().is_some() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let first = a.eg.and_then(|eg| eg.get_first());
    unported!(duel, first.is_none());
    let Some(first) = first else { return Ok(()) };
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE;
    c.register_flag_effect(duel, ID, reset, 0, 1);
    first.register_flag_effect(duel, ID + 1, reset, 0, 1);
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_flag_effect(duel, ID + 1) > 0
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let p = duel.get_turn_player();
    if let Some(tc) = g.get_first() {
        duel.get_control(tc, 1 - p)?;
    }
    Ok(())
}
