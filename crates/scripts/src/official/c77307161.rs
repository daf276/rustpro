// Trickstar Bloom
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 77307161;

pub const LISTED_SERIES: &[u16] = &[SET_TRICKSTAR];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_level_below(duel, 2) && c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp)
    }
}

// On the field with Trickstar monsters beside it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (298846, MZONE),
    (14365823, MZONE),
    (41302052, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, drtg);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(1 - tp, 1));
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, 1);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let mut seq = c.get_previous_sequence(duel);
    if !c.is_previous_controler(duel, tp) {
        seq += 16;
    }
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_set_card(duel, &[SET_TRICKSTAR])
        && c.is_previous_location(duel, LOCATION_MZONE)
        && zone >> seq & 1 != 0
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let (tp, zone) = (a.tp, a.e.get_handler(duel).get_linked_zone(duel));
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, tp, zone), 1, None)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(1 - tp, LOCATION_HAND, 0) > 0);
    }
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 0);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = duel.get_field_group_count(p, LOCATION_HAND, 0) * 200;
    duel.damage(p, dam as u32, REASON_EFFECT)?;
    Ok(())
}
