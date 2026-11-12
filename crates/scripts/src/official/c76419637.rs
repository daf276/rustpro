// CXyz Battleship Cherry Blossom
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 76419637;

pub const LISTED_NAMES: &[u32] = &[40424929];

// In the Extra Deck, Xyz Summoned with four Level 4 monsters, or on the field with Battlecruiser Dianthus attached by Xyz Import,
// with the opponent holding more cards in hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (5628232, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (40424929, MZONE | OPPONENT),
    (40424929, MZONE | OPPONENT),
    (2118022, HAND | OPPONENT),
    (2118022, HAND | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 4;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE | PHASE_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, dccon);
    e1.set_target(duel, dctg);
    e1.set_operation(duel, dcop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, damcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn dccon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_turn_player(1 - tp) && duel.get_field_group_count(tp, 0, LOCATION_HAND) > duel.get_field_group_count(tp, LOCATION_HAND, 0)
}

fn dctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 1);
    Ok(false)
}

fn dcop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_hand(1 - a.tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_code(duel, 40424929), 1, None)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let ct = duel.get_field_group_count(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ct as i32 * 300);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = duel.get_field_group_count(p, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.damage(p, ct as u32 * 300, REASON_EFFECT)?;
    Ok(())
}
