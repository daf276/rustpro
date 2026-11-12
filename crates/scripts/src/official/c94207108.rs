// Marincess Wonder Heart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 94207108;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(lc), sumtype, tp)
    }
}

// Link Summoned from WATER Normal Monsters (Psychic Kappa, Slime Toad), destroyed with Marincess Links (Blue Slug, Marbled Rock) in the Graveyard.
// Only "Marincess Battle Ocean" equips a monster to it, which is not ported, so the equipped Special Summon is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7892180, MZONE),
    (7892180, MZONE),
    (68638985, MZONE),
    (68638985, MZONE),
    (43735670, GRAVE),
    (5524387, GRAVE),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battle_target(duel).is_some());
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, spcon2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.get_equip_group(duel)
                .is_exists(duel, |c, duel| spfilter1(c, duel, e, tp), 1, None)
            && c.get_flag_effect(duel, ID) == 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_SZONE as i32);
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) || duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = c
        .get_equip_group(duel)
        .filter_select(duel, tp, |c, duel| spfilter1(c, duel, e, tp), 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let fid = c.get_field_id(duel);
        tc.register_flag_effect_full(duel, ID + 1, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
        e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e2.set_target_range(duel, 1, 0);
        e2.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        duel.register_effect(e2, tp);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e3.set_code(duel, EVENT_PHASE + PHASE_END);
        e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e3.set_count_limit(duel, 1, 0);
        e3.set_label(duel, fid.into());
        e3.set_label_object_group(duel, Some(tc.into()));
        e3.set_condition_mut(duel, eqcon);
        e3.set_operation(duel, eqop);
        duel.register_effect(e3, tp);
        c.create_effect_relation(duel, e3);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn eqcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_none_or(|tc| tc.get_flag_effect_label(duel, ID + 1) != Some(a.e.get_label(duel))) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_CARD, PLAYER_ALL, ID.into());
    let c = e.get_handler(duel);
    let tc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    let Some(tc) = tc else {
        return Ok(());
    };
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && duel.get_location_count(tp, LOCATION_SZONE) > 0 {
        duel.equip(tp, tc, c)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(c.into()));
        tc.register_effect(duel, e1);
    } else {
        duel.send_to_grave(tc, REASON_RULE)?;
    }
    Ok(())
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - a.tp && c.is_reason(duel, REASON_EFFECT) && c.get_previous_controler(duel) == a.tp)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && c.is_link_below(duel, 3) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter2(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter2(c, duel, e, tp));
    if let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
