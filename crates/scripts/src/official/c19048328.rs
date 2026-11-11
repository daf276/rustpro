// Chaofeng, Phantom of the Yang Zing
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 19048328;

// In the Extra Deck, Synchro Summoned with Chiwen (LIGHT) and Phantasm Spiral Dragon, with a Tuner and Wyrm Normal Monsters
// in the Deck, and Dark Hole and Raigeki with a LIGHT Wyrm on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (61488417, MZONE),
    (56649609, MZONE),
    (47643326, DECK),
    (89189982, DECK),
    (89189982, MZONE | OPPONENT),
    (53129443, HAND),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::CardHint(1014),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_operation(duel, regop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition_mut(duel, spcon);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let att = c
        .get_material(duel)
        .filter(duel, |c, duel| c.is_set_card(duel, &[SET_YANG_ZING]), None)
        .get_bitwise_or(duel, |c, duel| c.get_original_attribute(duel).into());
    e.set_label(duel, att as i64);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_label_object(duel, a.e.get_label_object(duel));
    c.register_effect(duel, e1);
    let att = a.e.get_label_object(duel).map_or(0, |e| e.get_label(duel)) as u32;
    for string in aux::get_attribute_strings(att) {
        c.register_flag_effect_full(duel, 0, RESET_EVENT | RESETS_STANDARD, EFFECT_FLAG_CLIENT_HINT, 1, 0, string);
    }
    Ok(())
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let att = e.get_label_object(duel).map_or(0, |e| e.get_label(duel)) as u32;
    re.is_monster_effect(duel) && att & re.get_handler(duel).get_original_attribute(duel) != 0
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_synchro_summoned(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn cfilter(c: CardId, duel: &Duel, p: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.get_original_attribute(duel) != 0
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.get_previous_controler(duel) == p
}

fn spcon(duel: &mut Duel, a: &Args) -> bool {
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, 1 - a.tp), None);
    if g.is_empty() {
        return false;
    }
    let att = g.get_bitwise_or(duel, |c, duel| c.get_original_attribute(duel).into());
    a.e.set_label(duel, att as i64);
    true
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, att: u32) -> bool {
    c.is_race(duel, RACE_WYRM)
        && att & c.get_original_attribute(duel) != 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let att = e.get_label(duel) as u32;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, att), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let att = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, att), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
