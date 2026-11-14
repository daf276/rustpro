// Sky Striker Mecha Modules - Multirole
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 24010609;

pub const LISTED_SERIES: &[u16] = &[SET_SKY_STRIKER];
// Sky Striker Spells in hand to activate and in the Graveyard to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (52340444, HAND),
    (52340444, HAND),
    (63166095, HAND),
    (52340444, GRAVE),
    (98338152, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID + 1) == 0);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_operation(duel, regop);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_CHAIN_NEGATED);
    e5.set_operation(duel, regop2);
    c.register_effect(duel, e5);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    if duel.get_flag_effect(tp, ID + 1) != 0 {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_operation(duel, actop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.is_spell_effect(duel)
    {
        duel.set_chain_limit(|_, _, rp, tp, _| tp == rp, 0);
    }
    Ok(())
}

fn is_own_sky_striker_spell(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| {
        re.get_handler(duel).is_set_card(duel, &[SET_SKY_STRIKER])
            && re.is_spell_effect(duel)
            && a.rp == a.tp
            && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
    })
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if is_own_sky_striker_spell(duel, a) {
        if let Some(flag) = c.get_flag_effect_label(duel, ID) {
            c.set_flag_effect_label(duel, ID, flag as u32 + 1);
        } else {
            c.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, 1, 0);
        }
    }
    Ok(())
}

fn regop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if is_own_sky_striker_spell(duel, a)
        && let Some(flag) = c.get_flag_effect_label(duel, ID)
        && flag > 0
    {
        c.set_flag_effect_label(duel, ID, flag as u32 - 1);
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER]) && c.is_spell(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = a.e.get_handler(duel).get_flag_effect_label(duel, ID);
    if chk == 0 {
        return Ok(ct.is_some_and(|ct| ct > 0) && duel.is_existing_matching_card(setfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(aux::necro_valley_filter(setfilter), tp, LOCATION_GRAVE, 0, None);
    let ct = c.get_flag_effect_label(duel, ID).unwrap_or(0);
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        let fc = sg.filter_count(duel, |c, duel| c.is_type(duel, TYPE_FIELD), None) as i32;
        let c1 = sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
        let c2 = sg.len() as i32;
        (
            c1 as i32 == c2 && fc <= 1 && c2 - fc <= ft,
            c1 as i32 != c2 || fc > 1 || c2 - fc > ft,
        )
    };
    let maxc = ct.min(i64::from(ft) + 1).max(0) as usize;
    let tg = aux::select_unselect_group(duel, g, 1, maxc, &rescon, tp, HINTMSG_SET)?;
    if tg.is_empty() || duel.sset(tp, tg)? == 0 {
        return Ok(());
    }
    for tc in tg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
