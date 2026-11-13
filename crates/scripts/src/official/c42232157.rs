// Stage of Destiny
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, SZONE};

const ID: u32 = 42232157;

// Set, with Normal Monsters (Kozaky, D.D. Trainer, Skull Servant) in the hand and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (99171160, HAND),
    (99171160, HAND),
    (86498013, HAND),
    (32274490, GRAVE),
    (32274490, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e0.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    e0.set_condition(duel, |duel, a| {
        let tp = a.e.get_handler_player(duel);
        duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None)
    });
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_label_object(duel, Some(e0));
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && !c.is_public(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(label_obj) = e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(true);
    }
    if label_obj.get_label(duel) > 0 {
        label_obj.set_label(duel, 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut mmz_count = duel.get_location_count(tp, LOCATION_MZONE);
    if mmz_count > 0 {
        mmz_count = mmz_count.min(5);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            mmz_count = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            mmz_count as u8,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?;
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
