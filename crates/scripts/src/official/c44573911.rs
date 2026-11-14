// R.B. Ga10 Pile Bunker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 44573911;

pub const LISTED_SERIES: &[u16] = &[SET_RB];
pub const LISTED_NAMES: &[u32] = &[32216688];
// The Brute Blues to attack while linked to this card, and the opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (32216688, MZONE),
    (32216688, MZONE),
    (32216688, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, super::c33438265::spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_cost(duel, Cost::pay_lp::<1500>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let (bc, oc) = duel.get_battle_monster(a.tp);
    oc.is_some() && bc.is_some_and(|bc| bc.is_code(duel, 32216688) && bc.get_linked_group(duel).contains(a.e.get_handler(duel)))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g.union(c.into())), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let mut g = duel.get_field_group(a.tp, 0, LOCATION_ONFIELD);
    if c.is_relate_to_effect(duel, a.e) && !g.is_empty() {
        g.insert(c);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
