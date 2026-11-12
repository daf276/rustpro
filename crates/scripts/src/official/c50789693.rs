// Armored Kappa
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 50789693;

// In the Extra Deck, Xyz Summoned with two Level 2 Normal Monsters (Doron), with the opponent's Elfnote Lucina to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, adop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, TIMING_BATTLE_PHASE, TIMING_BATTLE_PHASE);
    e2.set_condition(duel, btcon);
    e2.set_cost(duel, btcost);
    e2.set_operation(duel, btop);
    c.register_effect(duel, e2);
}

fn adop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let opt = duel.select_option(tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_code(duel, if opt == 0 { EFFECT_UPDATE_ATTACK } else { EFFECT_UPDATE_DEFENSE });
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn btcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_attacker().is_some_and(|bt| bt.is_controler(duel, tp)) || duel.get_attack_target().is_some_and(|bt| bt.is_controler(duel, tp))
}

fn btcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn btop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_value(duel, 1);
    duel.register_effect(e2, tp);
    Ok(())
}
