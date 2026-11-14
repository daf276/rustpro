// S-Force Rappa Chiyomaru
use super::c91864689::column_target;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 22180094;

pub const LISTED_SERIES: &[u16] = &[SET_S_FORCE];
// With S-Force monsters in the hand to banish and in the Deck to Special Summon, and opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (91864689, MZONE),
    (91864689, HAND),
    (91864689, DECK),
    (ID, HAND),
    (32295838, MZONE | OPPONENT),
    (32295838, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub const LISTED_NAMES: &[u32] = &[ID];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_target_range(duel, 0, LOCATION_MZONE);
    e1a.set_target_card(duel, |duel, e, c| {
        let column = column_target(duel, e, c);
        if column {
            duel.set_script_state(label_key(e), c.index() as i64 + 1);
        }
        column
    });
    e1a.set_value(
        duel,
        Value::Card(|duel, e, c| {
            let label = duel.get_script_state(label_key(e));
            let bc = duel.get_matching_group(|tc, _| tc.index() as i64 + 1 == label, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
            i32::from(bc.get_first().is_some_and(|bc| !bc.get_column_group(duel).is_contains(c)))
        }),
    );
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable_full(duel, a, chk, spcost, extracon));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

// The Lua keeps the card its target accepted as the effect's label object, which the value reads straight after.
fn label_key(e: EffectId) -> u64 {
    u64::from(ID) << 16 | e.index() as u64
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_S_FORCE]) && c.is_able_to_remove_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, spcostfilter, LOCATION_HAND)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_S_FORCE])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn extracon(duel: &Duel, _: EffectId, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, a.c.map(Into::into))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel)
            && duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_HAND)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}
