// Lev Shaddoll Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 34950192;

// In the hand to activate and on the field; El Shaddoll Meshahrail in the Extra Deck to summon with Rulkallos, a Fusion Monster of
// another Attribute, on the field to Tribute and in the Extra Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (32467459, EXTRA), (84330567, EXTRA), (84330567, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, effcost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[spcost, effcost]));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_set_card(duel, &[SET_SHADDOLL])),
    );
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_SHADDOLL])
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_fusion_monster(duel) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_EXTRA)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, attr: u32, cost_c: Option<CardId>) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL])
        && c.is_fusion_monster(duel)
        && c.is_attribute_except(duel, attr)
        && c.check_fusion_material(duel, None, None, u64::from(PLAYER_NONE))
        && duel.get_location_count_from_ex(tp, tp, cost_c.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let attr = c.get_attribute(duel);
    c.is_fusion_monster(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, attr, Some(c)), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let Some(sc) = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?.get_first() else {
        return Ok(false);
    };
    e.set_label(duel, i64::from(sc.get_attribute(duel)));
    duel.release(sc, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let label = a.e.get_label(duel);
    duel.set_target_param(label as u32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let attr: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, attr, None),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, None);
    if duel.special_summon_step(sc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    if duel.special_summon_complete()? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}
