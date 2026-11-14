// Enneacraft - Atil.SPIA
use super::c28454232::{flip_quick, register_counter, register_destroy, register_flip_flag, register_set_summon};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 71801447;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, both Pendulum Zones and face-down on the field, with the opponent's Sangan to destroy and Set Mystical Space
// Typhoons, which chain to each other.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_destroy::<ID>(duel, c);
    register_flip_flag::<ID, 1>(duel, c);
    let e3b = duel.create_effect(c);
    e3b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3b.set_code(duel, EVENT_CHAINING);
    e3b.set_range(duel, LOCATION_MZONE);
    e3b.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e3b.set_operation(duel, |duel, a| {
        if a.ep == 1 - a.tp {
            duel.set_chain_limit(|_, _, rp, tp, _| tp == rp, 0);
        }
        Ok(())
    });
    c.register_effect(duel, e3b);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_NEGATE + CATEGORY_DESTROY, 0);
    e4.set_condition(duel, negcon);
    e4.set_target(duel, negtg);
    e4.set_operation(duel, negop);
    c.register_effect(duel, e4);
    register_set_summon::<ID, 3>(duel, c);
}

// The chain links before this one that the opponent activated, with the effect each activated.
fn opponent_links(duel: &Duel, a: &Args) -> impl Iterator<Item = (u8, EffectId)> {
    let tp = a.tp;
    (1..=a.ev as u8).filter_map(move |i| {
        let te: Option<EffectId> = duel.get_chain_info(i, CHAININFO_TRIGGERING_EFFECT);
        let tgp: u8 = duel.get_chain_info(i, CHAININFO_TRIGGERING_PLAYER);
        te.filter(|_| tgp == 1 - tp).map(|te| (i, te))
    })
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_facedown(duel)
        && a.ep == 1 - a.tp
        && duel.get_current_chain_full(true) >= 3
        && opponent_links(duel, a).any(|(i, _)| duel.is_chain_negatable(i))
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let mut ng = CardSet::default();
    let mut dg = CardSet::default();
    for (i, te) in opponent_links(duel, a).collect::<Vec<_>>() {
        if duel.is_chain_negatable(i) {
            let tc = te.get_handler(duel);
            ng.insert(tc);
            if tc.is_destructable(duel) && tc.is_relate_to_effect(duel, te) {
                dg.insert(tc);
            }
        }
    }
    duel.set_target_card(dg);
    duel.set_operation_info(0, CATEGORY_NEGATE, Some(ng), ng.len() as u8, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, a.tp, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let mut negate_chk = false;
    let mut dg = CardSet::default();
    for (i, te) in opponent_links(duel, a).collect::<Vec<_>>() {
        if duel.negate_activation(i) {
            negate_chk = true;
            let tc = te.get_handler(duel);
            if tc.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, te) {
                tc.cancel_to_grave(duel);
                dg.insert(tc);
            }
        }
    }
    if negate_chk {
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
