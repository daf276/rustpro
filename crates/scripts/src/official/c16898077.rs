// Flying Fortress SKY FIRE
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 16898077;

// On the field with cards in both hands: the opponent Summons and Sets monsters on their turn, and its own hand pays the cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (7359741, HAND),
    (7359741, HAND),
    (7359741, HAND | OPPONENT),
    (7359741, HAND | OPPONENT),
    (7359741, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e3.set_condition(duel, damcon);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_MSET);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e5.set_condition(duel, damcon2);
    e5.set_target(duel, damtg2);
    e5.set_operation(duel, damop2);
    c.register_effect(duel, e5);
    let e6 = e5.clone_effect(duel);
    e6.set_code(duel, EVENT_SSET);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 2));
    e7.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e7.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e7.set_code(duel, EVENT_CHANGE_POS);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e7.set_condition(duel, damcon3);
    e7.set_target(duel, damtg3);
    e7.set_operation(duel, damop3);
    c.register_effect(duel, e7);
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
}

fn dfilter(c: CardId, duel: &Duel, e: Option<EffectId>, sp: u8) -> bool {
    c.get_summon_player(duel) == sp && e.is_none_or(|e| c.is_relate_to_effect(duel, e))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, eg) = (a.tp, a.eg.unwrap_or_default());
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| dfilter(c, duel, None, 1 - tp), 1, None));
    }
    let g = eg.filter(duel, |c, duel| dfilter(c, duel, None, 1 - tp), None);
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| dfilter(c, duel, Some(e), 1 - tp), None);
    if e.get_handler(duel).is_relate_to_effect(duel, e) && !g.is_empty() && duel.destroy(g, REASON_EFFECT)? != 0 {
        duel.damage(1 - tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}

fn damcon2(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && a.rp != a.tp
}

fn damtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, eg) = (a.tp, a.eg.unwrap_or_default());
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(eg), eg.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
    Ok(false)
}

fn damop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = a.eg.and_then(|eg| eg.get_first())
        && e.get_handler(duel).is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? != 0
    {
        duel.damage(1 - tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}

fn damcon3(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && a.rp != a.tp
}

fn sfilter(c: CardId, duel: &Duel, e: Option<EffectId>) -> bool {
    c.is_facedown(duel) && e.is_none_or(|e| c.is_relate_to_effect(duel, e))
}

fn damtg3(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, eg) = (a.tp, a.eg.unwrap_or_default());
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| sfilter(c, duel, None), 1, None));
    }
    let g = eg.filter(duel, |c, duel| sfilter(c, duel, None), None);
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
    Ok(false)
}

fn damop3(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| sfilter(c, duel, Some(e)), None);
    if e.get_handler(duel).is_relate_to_effect(duel, e) && !g.is_empty() && duel.destroy(g, REASON_EFFECT)? != 0 {
        duel.damage(1 - tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}
