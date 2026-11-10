// Max Metalmorph
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};
use engine::constants::MSG_EQUIP;

const ID: u32 = 89812483;

pub const LISTED_NAMES: &[u32] = &[CARD_MAX_METALMORPH];

// Set with a Level 8 Dragon (Galaxy-Eyes Photon Dragon) to Tribute and Red-Eyes Black Fullmetal Dragon, which cannot be Special
// Summoned otherwise, in the hand, Deck and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (80870883, HAND),
    (80870883, DECK),
    (80870883, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(80870883),
    Check::Message(MSG_EQUIP),
];

// The monsters whose script defines s.max_metalmorph_stats: (Level, Race).
fn max_metalmorph_stats(code: u32) -> Option<(i64, u64)> {
    match code {
        80870883 => Some((5, RACE_DRAGON)),
        27268998 => Some((5, RACE_FIEND)),
        27704731 => Some((5, RACE_WARRIOR)),
        _ => None,
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let (level, race) = (i64::from(c.get_level(duel)), c.get_race(duel));
    c.is_faceup(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |g, duel| spfilter(g, duel, e, tp, Some((level, race))),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, cost: Option<(i64, u64)>) -> bool {
    if !(c.is_monster(duel) && !c.is_summonable_card(duel) && c.lists_code(duel, CARD_MAX_METALMORPH)) {
        return false;
    }
    let stats = max_metalmorph_stats(c.get_original_code(duel));
    if c.is_can_be_special_summoned(duel, e, 0, tp, false, false) || stats.is_none() {
        return true;
    }
    let (Some((cost_level, cost_race)), Some((level, race))) = (cost, stats) else {
        return false;
    };
    c.is_can_be_special_summoned(duel, e, 0, tp, true, true) && cost_level >= level && cost_race & race > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let res = duel.check_release_group_cost(tp, |c, duel| costfilter(c, duel, e, tp), (1, 1), false, None, None);
        if res {
            e.set_label(duel, 1);
        }
        return Ok(res);
    }
    let g = duel.select_release_group_cost(tp, |c, duel| costfilter(c, duel, e, tp), (1, 1), false, None, None)?;
    let Some(rc) = g.get_first() else { return Ok(false) };
    let labels = [i64::from(rc.get_level(duel)), rc.get_race(duel) as i64];
    e.set_labels(duel, &labels);
    duel.release(rc, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let res = e.get_label(duel) == 1
            || (duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_matching_card(
                    |c, duel| spfilter(c, duel, e, tp, None),
                    tp,
                    LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                    0,
                    1,
                    None,
                ));
        e.set_label(duel, 0);
        return Ok(res);
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    let c = e.get_handler(duel);
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let cost = match e.get_labels(duel) {
        [level, race, ..] => Some((*level, *race as u64)),
        _ => None,
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|g, duel| spfilter(g, duel, e, tp, cost));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(tc, 0, tp, tp, true, true, POS_FACEUP)? <= 0 {
        return Ok(());
    }
    tc.complete_procedure(duel);
    if !(c.is_relate_to_effect(duel, e) && duel.select_yes_no(tp, aux::stringid(ID, 1))?) {
        return Ok(());
    }
    c.cancel_to_grave(duel);
    duel.break_effect()?;
    if !tc.equip_by_effect_and_limit_register(duel, e, tp, c, None, Some(true))? {
        return Ok(());
    }
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_EQUIP_LIMIT);
    e0.set_label_object_group(duel, Some(tc.into()));
    e0.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(e.get_label_object_group(duel) == Some(c.into()))),
    );
    e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 400);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(
        duel,
        Value::EffectPlayerCard(|duel, _, re, _, _| re.is_monster_effect(duel) || re.is_spell_effect(duel)),
    );
    e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e4.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e4.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, rp| {
            rp == 1 - e.get_handler_player(duel) && (re.is_monster_effect(duel) || re.is_spell_effect(duel))
        }),
    );
    c.register_effect(duel, e4);
    Ok(())
}
