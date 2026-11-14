// D/D/D Super Doom King Purple Armageddon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 84569886;

pub const LISTED_SERIES: &[u16] = &[SET_DDD];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DDD], Some(fc), sumtype, tp)); 2];
}

// On the field and in the Pendulum Zone with the D/D/D Fusion Monsters d'Arc and Genghis to battle the opponent's Elfnote Lucina,
// whose ATK Yami changes; the opponent's Raigeki destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, PZONE),
    (82956492, MZONE),
    (16006416, MZONE),
    (12580477, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (59197169, 0),
    (59197169, 0),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_CONFIRM);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atkcon1);
    e1.set_operation(duel, atkop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_CONFIRM);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon2);
    e3.set_operation(duel, atkop2);
    c.register_effect(duel, e3);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 3));
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_DESTROYED);
    e6.set_property(duel, EFFECT_FLAG_DELAY);
    e6.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
    });
    e6.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e6.set_operation(duel, penop);
    c.register_effect(duel, e6);
}

// The attacker and its battle target, swapped when the attacker is the opponent's.
fn battlers(duel: &Duel, tp: u8) -> (Option<CardId>, Option<CardId>) {
    let a = duel.get_attacker();
    let d = a.and_then(|a| a.get_battle_target(duel));
    match a.is_some_and(|a| a.is_controler(duel, 1 - tp)) {
        true => (d, a),
        false => (a, d),
    }
}

fn atkcon1(duel: &Duel, a: &Args) -> bool {
    let (Some(a), Some(d)) = battlers(duel, a.tp) else { return false };
    a.is_faceup(duel)
        && a.is_relate_to_battle(duel)
        && a.is_set_card(duel, &[SET_DDD])
        && a.is_type(duel, TYPE_FUSION)
        && d.is_faceup(duel)
        && d.is_relate_to_battle(duel)
        && d.get_attack(duel) > 0
        && a.get_controler(duel) != d.get_controler(duel)
}

fn atkop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let (_, d) = battlers(duel, a.tp);
    // The Lua indexes a nil battle target.
    unported!(duel, d.is_none());
    if let Some(d) = d
        && d.is_faceup(duel)
        && d.is_relate_to_battle(duel)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -1000);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        d.register_effect(duel, e1);
    }
    Ok(())
}

fn half_base_attack(duel: &Duel, c: CardId) -> i32 {
    (c.get_base_attack(duel) / 2).max(0)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = |c: CardId, duel: &Duel| c.is_position(duel, POS_FACEUP_ATTACK);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| half_base_attack(duel, tc));
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, atk);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let atk = half_base_attack(duel, tc);
        if duel.destroy(tc, REASON_EFFECT)? != 0 {
            duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn atkcon2(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(attacker) = duel.get_attacker() else { return false };
    let d = attacker.get_battle_target(duel);
    let a = if attacker.is_controler(duel, tp) { d } else { Some(attacker) };
    a.is_some_and(|a| a.is_controler(duel, 1 - tp) && a.get_attack(duel) != a.get_base_attack(duel))
}

fn atkop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(attacker) = duel.get_attacker() else { return Ok(()) };
    let tc = match attacker.is_controler(duel, a.tp) {
        true => attacker.get_battle_target(duel),
        false => Some(attacker),
    };
    if let Some(tc) = tc
        && tc.is_faceup(duel)
        && tc.get_attack(duel) != tc.get_base_attack(duel)
    {
        let base = tc.get_base_attack(duel);
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, base);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE_CAL);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
