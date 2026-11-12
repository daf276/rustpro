// Old Entity Cthugua
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 12948099;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Laval Coatl (Level 2 Tuner) with Fire Eye for a Synchro Summon from the Extra Deck, and Brotherhood of the Fire Fist - Tiger King
// (Rank 4) on both sides to shuffle, and Tripwire Beast (Level 4) with Gagaga Cowboy in the Extra Deck to Xyz Summon over it.
// The Fusion Material draw is only reported: no ported Fusion Monster takes it as a Material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (45439263, MZONE),
    (88435542, MZONE),
    (96381979, MZONE),
    (96381979, MZONE | OPPONENT),
    (45042329, MZONE),
    (12014404, EXTRA),
    (12014404, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesGranted(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, tdcon);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_condition(duel, |_, a| a.r == REASON_FUSION);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_condition(duel, |_, a| a.r == REASON_XYZ);
    e3.set_operation(duel, efop);
    c.register_effect(duel, e3);
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.get_rank(duel) == 4 && c.is_able_to_extra(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    }
    Ok(())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(rc);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, drtg);
    e1.set_operation(duel, stock::draw);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    if !rc.is_type(duel, TYPE_EFFECT) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_ADD_TYPE);
        e2.set_value(duel, TYPE_EFFECT as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect_full(duel, e2, true);
    }
    Ok(())
}
