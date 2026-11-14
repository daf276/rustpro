// Light End Sublimation Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 35614780;

// Copies in the hand and on the field, Black-Winged Dragon (a Level 8 Dragon) in the Extra Deck to banish, Karbonala
// Warrior with its M-Warrior materials to Fusion Summon, and the opponent's Dark Magicians to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (9012916, EXTRA),
    (9012916, EXTRA),
    (54541900, EXTRA),
    (54541900, EXTRA),
    (56342351, HAND),
    (92731455, HAND),
    (56342351, MZONE),
    (92731455, MZONE),
    (46986414, MZONE | OPPONENT),
    (46986414, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Params;

impl FusionSummonParams for Params {}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, hspcost);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|at| at.is_controler(duel, 1 - a.tp))
    });
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn hspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_remove_as_cost(duel)
}

fn hspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, hspcostfilter, LOCATION_EXTRA)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_DRAGON));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_attack_above(duel, 500) && c.is_defense_above(duel, 500));
    }
    let g = CardSet::from_iter([Some(c), duel.get_attacker()].into_iter().flatten());
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DEFCHANGE, Some(c.into()), 1, tp, -500);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.is_attack_above(duel, 500) && c.is_defense_above(duel, 500)) {
        return Ok(());
    }
    if c.update_attack(duel, -500, None, None, None) == -500
        && c.update_defense(duel, -500, None, None, None) == -500
        && let Some(at) = duel.get_attacker()
        && at.is_faceup(duel)
        && at.is_controler(duel, 1 - tp)
        && at.is_relate_to_battle(duel)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -1500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        at.register_effect(duel, e1);
    }
    Ok(())
}
