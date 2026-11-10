// Hecahands Breus
use super::c95132593::{add_indestructable_battle, add_self_special_summon};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 21637502;

// In the hand and on the field, with the opponent's Pot of Greed and Upstart Goblin to draw from the Deck, and cards in the
// opponent's hand to look at.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (55144522, HAND | OPPONENT),
    (70368879, HAND | OPPONENT),
    (70368879, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_indestructable_battle(duel, c);
    add_self_special_summon(duel, c, ID);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(());
    }
    let Some(sc) = g.random_select(duel, tp, 1).get_first() else {
        return Ok(());
    };
    duel.confirm_cards(tp, sc)?;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    duel.shuffle_hand(1 - tp)?;
    Ok(())
}
