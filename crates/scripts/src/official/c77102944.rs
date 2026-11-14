// Appliancer Laundry Dragon
use super::c3507053::{Procedure, register_cannot_be_link_material};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 77102944;

pub const LISTED_SERIES: &[u16] = &[SET_APPLIANCER];
// Link Summoned from an Appliancer monster, with opponent's monsters to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (78225596, EXTRA),
    (78225596, EXTRA),
    (78447174, MZONE),
    (78447174, MZONE),
    (78447174, MZONE),
    (4729591, MZONE),
    (81769387, MZONE),
    (78447174, HAND),
    (78447174, DECK),
    (3507053, EXTRA),
    (13597785, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
// A co-link needs Celtopus Link Summoned into the Extra Monster Zone with another Appliancer Link Monster in the zone it points to; random play rarely gets it.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    register_cannot_be_link_material(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_NO_BATTLE_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EVENT_BATTLED);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, rmcon);
    e4.set_operation(duel, rmop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_DESTROY);
    e5.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EVENT_BATTLED);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, descon);
    e5.set_operation(duel, desop);
    c.register_effect(duel, e5);
}

fn battle_target_of_opponent(duel: &Duel, a: &Args) -> Option<CardId> {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .filter(|bc| bc.is_controler(duel, 1 - a.tp))
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    battle_target_of_opponent(duel, a).is_some() && a.e.get_handler(duel).get_mutual_linked_group_count(duel) > 0
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel)
        && tc.is_relate_to_battle(duel)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    battle_target_of_opponent(duel, a).is_some() && a.e.get_handler(duel).get_mutual_linked_group_count(duel) == 0
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel)
        && tc.is_relate_to_battle(duel)
    {
        let atk = tc.get_base_attack(duel).max(0);
        if duel.destroy(tc, REASON_EFFECT)? > 0 {
            duel.damage(1 - tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
