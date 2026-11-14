// Mitsurugi Tempest
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE, SZONE};

const ID: u32 = 49604192;

pub const LISTED_NAMES: &[u32] = &[19899073, 55397172, 13332685];
pub const LISTED_SERIES: &[u16] = &[SET_MITSURUGI];
// Set, with the three Mitsurugi Ritual Monsters on the field to release and Mitsurugi Ritual in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (81560239, GRAVE),
    (19899073, MZONE),
    (55397172, MZONE),
    (13332685, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(rmconfilter, a.tp, LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_cost(duel, rmcost);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
}

fn rmconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MITSURUGI]) && c.is_ritual_spell(duel)
}

fn rmcostrescon(duel: &Duel, sg: CardSet, _: u8, _: CardSet) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_original_code_rule(duel))) == 3,
        false,
    )
}

fn ritual_monster(c: CardId, duel: &Duel) -> bool {
    LISTED_NAMES.contains(&c.get_original_code_rule(duel))
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_release_group_cost(a.tp, ritual_monster, (3, 3), false, Some(&rmcostrescon), None));
    }
    let g = duel.select_release_group_cost(a.tp, ritual_monster, (3, 3), false, Some(&rmcostrescon), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn removable(p: u8) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| c.is_able_to_remove_full(duel, Some(p), POS_FACEUP, REASON_EFFECT)
}

const LOCATIONS: u32 = LOCATION_ONFIELD | LOCATION_HAND | LOCATION_EXTRA | LOCATION_GRAVE;

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.is_player_affected_by_effect(1 - tp, 30459350).is_none()
        && duel.is_existing_matching_card(removable(1 - tp), tp, 0, LOCATIONS, 8, None))
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p = 1 - a.tp;
    duel.hint(HINT_SELECTMSG, p, HINTMSG_REMOVE);
    let g = duel.select_matching_card(p, removable(p), p, LOCATIONS, 0, 8, 8, None)?;
    if g.len() == 8 {
        duel.remove_full(g, POS_FACEUP, REASON_RULE, PLAYER_NONE, Some(p))?;
    }
    Ok(())
}
