// Dominus Purge
use crate::prelude::*;

use super::c40366667::{disop, opponent_controls_card, set_hand_label_and_infos};

const ID: u32 = 97045737;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, discond);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop::<ID, { ATTRIBUTE_DARK | ATTRIBUTE_WATER | ATTRIBUTE_FIRE }>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, opponent_controls_card);
    c.register_effect(duel, e2);
}

fn discond(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    if !duel.is_chain_disablable(ev) {
        return false;
    }
    if a.re
        .is_some_and(|re| re.is_has_category(duel, CATEGORY_SEARCH) || re.is_has_category(duel, CATEGORY_DRAW))
    {
        return true;
    }
    let infos = [
        duel.get_operation_info(ev, CATEGORY_TOHAND),
        duel.get_possible_operation_info(ev, CATEGORY_TOHAND),
    ];
    let mut g = CardSet::default();
    let mut loc = 0;
    for info in infos.into_iter().flatten() {
        if let Some(cards) = info.op_cards {
            g.merge(cards);
        }
        loc |= info.op_param as u32;
    }
    loc & LOCATION_DECK > 0 || (!g.is_empty() && g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    set_hand_label_and_infos(duel, a);
    Ok(false)
}
