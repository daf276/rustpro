// Bujintei Tsukuyomi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 73289035;

pub const LISTED_SERIES: &[u16] = &[SET_BUJIN];
// In the Extra Deck, Xyz Summoned with Level 4 LIGHT monsters (Fiend Reflection #2), with Yamato in the Graveyard and the
// opponent's Mystical Space Typhoon, Dark Hole, and Raigeki to destroy it with materials.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2863439, MZONE),
    (2863439, MZONE),
    (32339440, GRAVE),
    (32339440, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_condition_mut(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2) && duel.get_field_group_count(tp, LOCATION_HAND, 0) > 0);
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 0, LOCATION_HAND as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    if !g.is_empty()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) > 0
    {
        duel.draw(tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let ct = c.get_overlay_count(duel);
    a.e.set_label(duel, ct as i64);
    a.rp != a.tp
        && a.r & REASON_EFFECT != 0
        && ct > 0
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.get_level(duel) == 4
        && c.is_set_card(duel, &[SET_BUJIN])
        && c.is_race(duel, RACE_BEASTWARRIOR)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let mut ct = (e.get_label(duel) as i32).min(ft);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    if ft < g.len() as i32 {
        return Ok(());
    }
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
