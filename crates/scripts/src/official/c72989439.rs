// Black Luster Soldier - Envoy of the Beginning
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72989439;

// Copies in the hand to Special Summon with a LIGHT and a DARK monster from the Graveyard, and on the field to banish and
// to battle weak Sangan with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (CARD_DARK_MAGICIAN, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon1);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, rmcost);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_condition(duel, atcon);
    e3.set_operation(duel, atop);
    c.register_effect(duel, e3);
}

pub fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let atchk1 = |c: CardId, duel: &Duel| {
        c.is_attribute(duel, ATTRIBUTE_LIGHT) && sg.filter_count(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_DARK), Some(c.into())) == 1
    };
    (
        duel.get_mzone_count(tp, Some(sg)) >= 1 && sg.is_exists(duel, atchk1, 1, None),
        false,
    )
}

fn spfilter1(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_attribute(duel, att) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

pub fn spcon1(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let rg1 = duel.get_matching_group(
        |c, duel| spfilter1(c, duel, ATTRIBUTE_LIGHT),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        None,
    );
    let rg2 = duel.get_matching_group(
        |c, duel| spfilter1(c, duel, ATTRIBUTE_DARK),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        None,
    );
    let mut rg = rg1;
    rg.merge(rg2);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    ft > -2
        && !rg1.is_empty()
        && !rg2.is_empty()
        && aux::select_unselect_group_check(duel, rg, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp))
}

pub fn sptg1(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(
        |c, duel| spfilter1(c, duel, ATTRIBUTE_LIGHT + ATTRIBUTE_DARK),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        None,
    );
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, tp),
        tp,
        HINTMSG_REMOVE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

pub fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_attack_announced_count(duel) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect_full(duel, e1, true);
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c)
        && aux::bdocon(duel, a)
        && !c.has_flag_effect(duel, ID)
        && duel.check_mut(|duel| c.can_chain_attack(duel))
}

fn atop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.chain_attack();
    Ok(())
}
