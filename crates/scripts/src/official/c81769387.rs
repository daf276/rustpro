// Appliancer Propelion
use super::c3507053::{Procedure, register_cannot_be_link_material};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 81769387;

// Link Summoned from an Appliancer monster, with Appliancer Link monsters to co-link with and opponent's monsters to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (78225596, EXTRA),
    (78225596, EXTRA),
    (78447174, MZONE),
    (78447174, MZONE),
    (78447174, MZONE),
    (4729591, MZONE),
    (77102944, MZONE),
    (78447174, HAND),
    (78447174, DECK),
    (3507053, EXTRA),
    (13597785, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
// Its effects need it or a co-linked Link Monster to attack a monster, which random play rarely does.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    register_cannot_be_link_material(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_condition(duel, atkcon2);
    c.register_effect(duel, e4);
}

// The attacker and the target, the player's monster first.
fn attack_pair(duel: &Duel, tp: u8) -> Option<(CardId, CardId)> {
    let (a, b) = (duel.get_attacker()?, duel.get_attack_target()?);
    Some(if a.is_controler(duel, 1 - tp) { (b, a) } else { (a, b) })
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    attack_pair(duel, tp).is_some_and(|(a1, b)| {
        a.e.get_handler(duel).get_mutual_linked_group_count(duel) > 0
            && a1.is_controler(duel, tp)
            && b.is_controler(duel, 1 - tp)
            && duel.is_phase(PHASE_DAMAGE_CAL)
    })
}

fn atkcon2(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    attack_pair(duel, tp).is_some_and(|(a1, b)| {
        c.get_mutual_linked_group_count(duel) == 0 && a1 == c && b.is_controler(duel, 1 - tp) && duel.is_phase(PHASE_DAMAGE_CAL)
    })
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (Some(attacker), Some(target)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return Ok(());
    };
    let tc = if target.is_controler(duel, 1 - tp) { target } else { attacker };
    if tc.is_relate_to_battle(duel) && tc.is_faceup(duel) && tc.is_controler(duel, 1 - tp) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE_CAL);
        e1.set_value(duel, 0);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
