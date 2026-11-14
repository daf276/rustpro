// Gladiator Beast Augustus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 7573135;

// Summoned from the hand with Gladiator Beast Gistel or by a tag-out from the Deck, with Hoplomus in the hand and Deck to
// summon in turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, DECK),
    (26582143, HAND),
    (26582143, HAND),
    (4253484, MZONE),
    (4253484, HAND),
    (4253484, DECK),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, aux::gbspcon);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_deck);
    e2.set_target(duel, |duel, a, chk, _| {
        super::c57731460::sptg(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp))
    });
    e2.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| filter(c, duel, a.e, a.tp), 113, false, 0)
    });
    c.register_effect(duel, e2);
}

fn hspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned_full(duel, e, 113, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| hspfilter(c, duel, e, tp), LOCATION_HAND)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| hspfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.special_summon(tc, 113, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        let c = e.get_handler(duel);
        let fid = c.get_field_id(duel);
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
        let code = tc.get_original_code(duel);
        tc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_count_limit(duel, 1, 0);
        e1.set_label(duel, fid.into());
        e1.set_label_object_group(duel, Some(g));
        e1.set_condition_mut(duel, retcon);
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.and_then(|tc| tc.get_flag_effect_label(duel, ID)) != Some(a.e.get_label(duel)) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = a.e.get_label_object_group(duel).unwrap_or_default();
    duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 113, tp, false, false)
}
