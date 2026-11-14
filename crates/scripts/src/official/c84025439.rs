// Super Quantal Mech King Great Magnus
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 84025439;

pub const LISTED_SERIES: &[u16] = &[SET_SUPER_QUANT, SET_SUPER_QUANTAL_MECH_BEAST];
// On the field with Xyz Import to attach two different opponent's monsters (Sangan, Dragon Rebellion) for the shuffle. The
// revive needs a third Super Quantal Mech Beast Xyz Monster, whose ports are in another batch, so it is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (16195942, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_main_phase() && effcon::<2>(duel, a));
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, effcon::<4>);
    e3.set_value(duel, Value::Effect(|duel, _, te| !te.is_card_setcode(duel, SET_SUPER_QUANT)));
    c.register_effect(duel, e3);
    let e4a = duel.create_effect(c);
    e4a.set_type(duel, EFFECT_TYPE_FIELD);
    e4a.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4a.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e4a.set_range(duel, LOCATION_MZONE);
    e4a.set_target_range(duel, 0, 1);
    e4a.set_condition(duel, effcon::<6>);
    e4a.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_DECK));
    c.register_effect(duel, e4a);
    let e4b = e4a.clone_effect(duel);
    e4b.set_code(duel, EFFECT_CANNOT_DRAW);
    c.register_effect(duel, e4b);
}

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 3;
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANTAL_MECH_BEAST])
        && c.is_xyz_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn class_count_at_least_3(duel: &Duel, g: CardSet) -> bool {
    g.len() >= 3 && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.get_location_count(tp, LOCATION_MZONE) < 3 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            return Ok(false);
        }
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
        return Ok(class_count_at_least_3(duel, g));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 3 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if class_count_at_least_3(duel, g) {
        let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
        if sg.len() == 3 {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn effcon<const MATERIAL_COUNT: usize>(duel: &Duel, a: &Args) -> bool {
    let g = a.e.get_handler(duel).get_overlay_group(duel);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= MATERIAL_COUNT
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
