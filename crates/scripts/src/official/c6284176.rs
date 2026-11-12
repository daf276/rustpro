// Kanzashi the Rikka Queen
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6284176;

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters, with Level 5 Normal Monsters in the hand to Tribute Summon, a Plant to Tribute
// in the hand, and the opponent's Raigeki to destroy the Plants.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (8353769, MZONE),
    (9540040, MZONE),
    (2971090, HAND),
    (4035199, HAND),
    (13193642, HAND),
    (27520594, MZONE),
    (6631034, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
// The destroy replacement has no check of its own: its prompt is the closest message.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_RELEASE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, repop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) || c.get_previous_type_on_field(duel) & TYPE_MONSTER == TYPE_MONSTER
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, spconfilter, 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_GRAVE,
        LOCATION_GRAVE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e2);
        let e3 = e1.clone_effect(duel);
        e3.set_code(duel, EFFECT_CHANGE_RACE);
        e3.set_value(duel, RACE_PLANT as i32);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        tc.register_effect(duel, e3);
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_race(duel, RACE_PLANT)
        && c.is_reason(duel, REASON_EFFECT)
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_PLANT)
        && c.is_controler(duel, tp)
        && c.is_releasable_by_effect_full(duel, Some(e), None)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED + STATUS_BATTLE_DESTROYED)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ct = a.eg.unwrap_or_default().filter_count(duel, |c, duel| repfilter(c, duel, tp), None);
        return Ok(ct > 0
            && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
            && duel.get_flag_effect(tp, ID) == 0);
    }
    if !duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let tg = duel.select_matching_card(
        tp,
        |c, duel| cfilter(c, duel, e, tp),
        tp,
        LOCATION_MZONE | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    duel.set_target_card(tg);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(true)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 1 - a.tp, u64::from(ID));
    if let Some(tc) = duel.get_first_target() {
        duel.release(tc, REASON_EFFECT + REASON_REPLACE)?;
    }
    Ok(())
}
