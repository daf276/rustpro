// Whitebeard, the Plunder Patroll Helm
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 31374201;

pub const LISTED_SERIES: &[u16] = &[SET_PLUNDER_PATROLL];
pub const LISTED_NAMES: &[u32] = &[ID];

// Plunder Patrollship Brann in the Extra Deck to summon and equip to, FIRE monsters on the opponent's side for its Attribute, Goldenhair in
// the Deck to Special Summon, and Raigeki in the opponent's hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (94253655, EXTRA),
    (13179332, MZONE | OPPONENT),
    (13179332, MZONE | OPPONENT),
    (81344070, DECK),
    (81344070, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND | LOCATION_MZONE)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, att: u32) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL])
        && c.is_attribute(duel, att)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE))
}

fn attributes(duel: &Duel, tp: u8) -> u32 {
    let mut att = 0;
    for gc in duel.get_matching_group(cfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, None) {
        att |= gc.get_attribute(duel);
    }
    att
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let att = attributes(duel, tp);
    if chk == 0 {
        return Ok(att > 0
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp, att), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let att = attributes(duel, tp);
    if att == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = duel
        .select_matching_card(tp, |c, duel| filter(c, duel, e, tp, att), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let c = e.get_handler(duel);
        if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
            return Ok(());
        }
        duel.equip(tp, c, tc)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
        && !c.is_code(duel, ID)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if g.get_first().is_some() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, super::c81344070::splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
