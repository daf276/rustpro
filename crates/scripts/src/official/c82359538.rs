// Enneacraft - Aiza.LEON
use super::c28454232::{flip_quick, register_counter, register_destroy, register_flip_flag, register_set_summon};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 82359538;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone and face-down on the field, with Pot of Greed for the opponent's Ash Blossom to chain to, their
// own Pot of Greed to add cards and Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (55144522, HAND),
    (55144522, HAND),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_destroy::<ID>(duel, c);
    register_set_summon::<ID, 1>(duel, c);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_TOHAND, 0);
    e4.set_condition(duel, rthcon);
    e4.set_target(duel, rthtg);
    e4.set_operation(duel, rthop);
    c.register_effect(duel, e4);
    register_flip_flag::<ID, 3>(duel, c);
    let e5b = duel.create_effect(c);
    e5b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5b.set_code(duel, EVENT_TO_HAND);
    e5b.set_range(duel, LOCATION_MZONE);
    e5b.set_condition(duel, damcon);
    e5b.set_operation(duel, damop);
    c.register_effect(duel, e5b);
    let e5c = duel.create_effect(c);
    e5c.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5c.set_property(duel, EFFECT_FLAG_DELAY);
    e5c.set_code(duel, EVENT_CHAIN_SOLVED);
    e5c.set_range(duel, LOCATION_MZONE);
    e5c.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e5c.set_operation(duel, chainsolvedop);
    c.register_effect(duel, e5c);
}

fn rthcon(duel: &Duel, a: &Args) -> bool {
    let ch = i32::from(duel.get_current_chain()) - 1;
    a.e.get_handler(duel).is_facedown(duel)
        && ch > 0
        && a.ep == 1 - a.tp
        && duel.get_chain_info::<u8>(ch as u8, CHAININFO_TRIGGERING_CONTROLER) == a.tp
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 3, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).has_flag_effect(duel, ID)
        && a.rp == 1 - a.tp
        && a.r & REASON_EFFECT > 0
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - a.tp), 1, None))
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct =
        a.eg.map_or(0, |eg| eg.filter_count(duel, |c, duel| c.is_controler(duel, 1 - tp), None));
    if duel.is_chain_solving() {
        let c = a.e.get_handler(duel);
        for _ in 0..ct {
            c.register_flag_effect(duel, ID + 1, RESET_CHAIN, 0, 1);
        }
    } else {
        duel.hint(HINT_CARD, 1 - tp, ID.into());
        duel.damage(1 - tp, ct as u32 * 900, REASON_EFFECT)?;
    }
    Ok(())
}

fn chainsolvedop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = c.get_flag_effect(duel, ID + 1);
    if ct > 0 {
        duel.hint(HINT_CARD, 1 - a.tp, ID.into());
        duel.damage(1 - a.tp, ct as u32 * 900, REASON_EFFECT)?;
        c.reset_flag_effect(duel, ID + 1);
    }
    Ok(())
}
