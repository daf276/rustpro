// Gold Pride - Nytro Head
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 59900655;
const NYTRO_TOKEN: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[NYTRO_TOKEN];

// Copies in the hand and on the field for the opponent's Standby Phase Token and to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(NYTRO_TOKEN),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_lp(a.tp) < duel.get_lp(1 - a.tp));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, tkntg);
    e2.set_operation(duel, tknop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| duel.is_main_phase() && duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 0, 0, 8, RACE_PYRO, ATTRIBUTE_FIRE);
    duel.is_player_can_special_summon_monster_full(tp, NYTRO_TOKEN, stats, POS_FACEUP, 1 - tp, 0)
}

fn tkntg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0 && can_summon_token(duel, tp),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tknop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(1 - tp, LOCATION_MZONE) > 0 && can_summon_token(duel, tp) {
        let Some(token) = duel.create_token(tp, NYTRO_TOKEN) else {
            return Ok(());
        };
        if duel.special_summon_step(token, 0, tp, 1 - tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3312);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            token.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}

pub fn cfilter(c: CardId, duel: &Duel, tc: CardId, seq: u32) -> bool {
    if tc.is_location(duel, LOCATION_SZONE) && c.is_controler(duel, tc.get_controler(duel)) {
        if c.is_location(duel, LOCATION_MZONE) {
            return c.is_sequence(duel, seq);
        }
        return true;
    }
    if tc.is_location(duel, LOCATION_MZONE) {
        if c.is_location(duel, LOCATION_SZONE) {
            return tc.is_in_main_mzone(duel, None)
                && tc.get_column_group(duel).is_contains(c)
                && c.is_controler(duel, tc.get_controler(duel));
        }
        if c.is_location(duel, LOCATION_MZONE) {
            if c.is_in_extra_mzone(duel) || tc.is_in_extra_mzone(duel) {
                return tc.get_column_group(duel).is_contains(c);
            }
            return c.get_sequence(duel).abs_diff(seq) == 1 && c.is_controler(duel, tc.get_controler(duel));
        }
    }
    false
}

fn tokenfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, NYTRO_TOKEN)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && tokenfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tokenfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let Some(tc) = duel
        .select_target(tp, tokenfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let seq = tc.get_sequence(duel);
    let mut g = tc
        .get_column_group_full(duel, 1, 1)
        .filter(duel, |c, duel| cfilter(c, duel, tc, seq), None);
    g.insert(tc);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_type(duel, TYPE_TOKEN)
    {
        let seq = tc.get_sequence(duel);
        let mut g = tc
            .get_column_group_full(duel, 1, 1)
            .filter(duel, |c, duel| cfilter(c, duel, tc, seq), None);
        g.insert(tc);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
