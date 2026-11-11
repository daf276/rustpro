// Gorz the Emissary of Darkness
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 44330098;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// In the hand, with an empty field, Blue-Eyes White Dragon (a Normal Monster) and Hinotama of the opponent to deal the damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (46130346, HAND | OPPONENT),
    (46130346, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_DAMAGE);
    e1.set_condition(duel, sumcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 2
    });
    e2.set_target(duel, sumtg2);
    e2.set_operation(duel, sumop2);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e3.set_target(duel, sumtg3);
    e3.set_operation(duel, stock::damage);
    e3.set_label_object(duel, Some(e1));
    c.register_effect(duel, e3);
}

fn sumcon(duel: &Duel, a: &Args) -> bool {
    a.ep == a.tp
        && a.tp != a.rp
        && !duel.is_existing_matching_card(
            |c, duel| !c.is_status(duel, STATUS_LEAVE_CONFIRMED),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let sumtype = if a.r & REASON_BATTLE != 0 { 2 } else { 1 };
    if duel.special_summon(c, sumtype, tp, tp, false, false, POS_FACEUP)? != 0 {
        e.set_label(duel, a.ev.into());
    }
    Ok(())
}

fn sumtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn sumop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let val = e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel)) as i32;
    let stats = MonsterStats::new(0, TYPES_TOKEN, -2, -2, 7, RACE_FAIRY, ATTRIBUTE_LIGHT);
    if !duel.is_player_can_special_summon_monster(tp, ID + 1, stats) {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, ID + 1) else {
        return Ok(());
    };
    for code in [EFFECT_SET_ATTACK, EFFECT_SET_DEFENSE] {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_value(duel, val);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
        token.register_effect(duel, e1);
    }
    duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn sumtg3(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let d = a.e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel)) as u32;
    duel.set_target_player(1 - tp);
    duel.set_target_param(d);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, d as i32);
    Ok(false)
}
