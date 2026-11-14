// Kashtira Riseheart
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 31149212;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_KASHTIRA];
// Copies in the hand with Kashtira Fenrir on the field, and Kashtira Fenrir and Unicorn in the Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (32909498, MZONE),
    (32909498, MZONE),
    (32909498, DECK),
    (68304193, DECK),
    (68304193, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e2.set_cost(duel, rmcost);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        ge1.set_code(duel, EVENT_SUMMON_SUCCESS);
        ge1.set_label(duel, i64::from(ID));
        ge1.set_operation(duel, aux::sumreg);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        duel.register_effect(ge2, 0);
    });
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_KASHTIRA]));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    xyz_only_lock(duel, c, tp, ID);
    Ok(())
}

// The Extra Deck lock of Kashtira Riseheart and Kashtiratheosis.
pub fn xyz_only_lock(duel: &mut Duel, c: CardId, tp: u8, id: u32) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(id, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !c.is_type(duel, TYPE_XYZ) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_XYZ));
}

fn rmcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_KASHTIRA]) && !c.is_code(duel, ID) && c.is_able_to_remove_as_cost(duel)
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, rmcostfilter, LOCATION_DECK)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_decktop_group(1 - tp, 3);
    if chk == 0 {
        return Ok(rg.filter_count(
            duel,
            |c, duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT),
            None,
        ) == 3
            && !a.e.get_handler(duel).is_level(duel, &[7]));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), 3, 1 - tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_REMOVED) && c.is_position(duel, POS_FACEDOWN)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_decktop_group(1 - a.tp, 3);
    if g.is_empty() {
        return Ok(());
    }
    duel.disable_shuffle_check(true);
    if duel.remove(g, POS_FACEDOWN, REASON_EFFECT)? > 0 && duel.get_operated_group().is_exists(duel, rmfilter, 1, None) {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 7);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
