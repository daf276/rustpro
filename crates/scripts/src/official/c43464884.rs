// Three Trolling Trolls
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 43464884;

// Copies in the hand to Normal Summon and on the field, and Battle Ox, a Level 4 Beast-Warrior Normal Monster, in the Graveyard
// to copy and in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (5053103, GRAVE),
    (5053103, HAND),
    (5053103, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, cgtg);
    e1.set_operation(duel, cgop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, cost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

fn counterfilter(duel: &Duel, c: CardId) -> bool {
    c.is_race(duel, RACE_BEASTWARRIOR)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_BEASTWARRIOR));
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(false)
}

fn cgfilter(c: CardId, duel: &Duel, mc: CardId) -> bool {
    c.is_race(duel, RACE_BEASTWARRIOR)
        && c.has_level(duel)
        && !(c.get_level(duel) == mc.get_level(duel) && c.is_attribute(duel, mc.get_attribute(duel)))
}

fn cgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && cgfilter(chkc, duel, c));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| cgfilter(tc, duel, c), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |tc, duel| cgfilter(tc, duel, c), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn cgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
    {
        let (lv, att) = (tc.get_level(duel), tc.get_attribute(duel));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e2.set_value(duel, att as i32);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_race(duel, RACE_BEASTWARRIOR)
        && c.is_level(duel, &[mc.get_level(duel)])
        && c.is_attribute(duel, mc.get_attribute(duel))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|tc, duel| spfilter(tc, duel, e, tp, c), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |tc, duel| spfilter(tc, duel, e, tp, c), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
