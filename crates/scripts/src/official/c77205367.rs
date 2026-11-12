// Number C96: Dark Storm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 77205367;
pub const XYZ_NUMBER: u32 = 96;
const DARK_MIST: u32 = 55727845;

pub const LISTED_NAMES: &[u32] = &[DARK_MIST];

// In the Extra Deck, Xyz Summoned with four Level 3 DARK Normal Monsters, and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32864, MZONE),
    (2468169, MZONE),
    (7670542, MZONE),
    (8944575, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
// The ATK effect needs "Number 96: Dark Mist" attached, which a Level 3 Xyz procedure never gives.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::Resolves)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 4;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ALSO_BATTLE_DAMAGE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let overlay = a.e.get_handler(duel).get_overlay_group(duel);
    overlay.is_exists(duel, |c, duel| c.is_code(duel, DARK_MIST), 1, None)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        let (attacker, target) = (duel.get_attacker(), duel.get_attack_target());
        unported!(duel, target == Some(c) && attacker.is_none());
        let attacking = attacker == Some(c) && target.is_some_and(|at| at.is_faceup(duel) && at.get_attack(duel) > 0);
        let attacked = target == Some(c) && attacker.is_some_and(|at| at.get_attack(duel) > 0);
        return Ok((attacking || attacked) && !c.is_status(duel, STATUS_CHAINING));
    }
    duel.set_target_card(c.get_battle_target(duel).map(CardSet::from).unwrap_or_default());
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.get_attack(duel) > 0
    {
        let atk = tc.get_base_attack(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_value(duel, atk);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    }
    Ok(())
}
