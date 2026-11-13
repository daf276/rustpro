// Sleepy Scapegoats
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_YESNO;

const ID: u32 = 3129527;
const TOKEN_SCAPEGOAT: u32 = ID + 1;
const SWIFTWIND_PANTHER_WARRIOR: u32 = 77482666;

pub const LISTED_NAMES: &[u32] = &[TOKEN_SCAPEGOAT, SWIFTWIND_PANTHER_WARRIOR, CARD_DARK_TIME_WIZARD];

// In the hand against the opponent's Sangan, with Panthers in the Deck to summon and on the field to protect from Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (77482666, DECK),
    (77482666, DECK),
    (77482666, DECK),
    (77482666, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(3129528),
    Check::SpecialSummonedCode(77482666),
    Check::Message(MSG_SELECT_YESNO),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_BEAST, ATTRIBUTE_EARTH);
    duel.is_player_can_special_summon_monster_full(tp, TOKEN_SCAPEGOAT, stats, POS_FACEUP_DEFENSE, tp, 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, SWIFTWIND_PANTHER_WARRIOR) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut mmz_count = duel.get_location_count(tp, LOCATION_MZONE);
    if mmz_count > 0 && can_summon_token(duel, tp) {
        mmz_count = mmz_count.min(4);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            mmz_count = 1;
        }
        if mmz_count > 1 {
            mmz_count = duel.announce_number_range(tp, 1, mmz_count as u32)? as i32;
        }
    }
    let c = e.get_handler(duel);
    for _ in 0..mmz_count {
        let Some(token) = duel.create_token(tp, TOKEN_SCAPEGOAT) else {
            continue;
        };
        if duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3304);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_UNRELEASABLE_SUM);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            token.register_effect_full(duel, e1, true);
        }
    }
    if duel.special_summon_complete()? > 0 {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EFFECT_DESTROY_REPLACE);
        e2.set_target(duel, reptg);
        e2.set_operation(duel, repop);
        e2.set_value(
            duel,
            Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
        );
        duel.register_effect(e2, tp);
        if duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_target_range(duel, 1, 0);
    e3.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_fusion_monster(duel)
    });
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.lists_code(duel, CARD_DARK_TIME_WIZARD)
        && c.is_on_field(duel)
        && c.is_controler(duel, tp)
        && c.is_faceup(duel)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_original_code(duel, TOKEN_SCAPEGOAT)
        && c.is_owner(duel, tp)
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None))
            && duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    if duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
        let g = duel.select_matching_card(tp, |c, duel| desfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
        let sc = g.get_first();
        unported!(duel, sc.is_none());
        duel.hint_selection(g);
        e.set_label_object_group(duel, Some(g));
        if let Some(sc) = sc {
            sc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
        }
        return Ok(true);
    }
    Ok(false)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let sc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, sc.is_none());
    if let Some(sc) = sc {
        sc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(sc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
