// Sunvine Healer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 65563871;

pub const LISTED_SERIES: &[u16] = &[SET_SUNAVALON];

pub struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, st: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PLANT, Some(lc), st, tp) && c.is_type_full(duel, TYPE_NORMAL, Some(lc), st, tp)
    }
}

// On the field and in the Extra Deck, Link Summoned with Sunseed Genius Loci, with Sunavalon Dryas on the field to recover for.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (27520594, MZONE),
    (27520594, MZONE),
    (93896655, MZONE),
    (93896655, MZONE),
    (93896655, MZONE | OPPONENT),
    (27520594, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_LEAVE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_target(duel, rectg);
    e2.set_operation(duel, recop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_RECOVER);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_condition(duel, reccon2);
    e3.set_target(duel, rectg2);
    e3.set_operation(duel, recop2);
    c.register_effect(duel, e3);
}

pub fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP)
        && c.is_set_card(duel, &[SET_SUNAVALON])
        && c.is_type(duel, TYPE_LINK)
        && c.get_previous_controler(duel) == tp
        && c.is_reason(duel, REASON_EFFECT)
}

pub fn descon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

pub fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    Ok(false)
}

pub fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn recfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SUNAVALON]) && c.is_type(duel, TYPE_LINK)
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && recfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(recfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, recfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let link = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_link(duel));
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, link as i32 * 300);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
        && tc.get_link(duel) > 0
    {
        duel.recover(a.tp, tc.get_link(duel) * 300, REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_type(duel, TYPE_LINK) && c.is_race(duel, RACE_PLANT)
}

fn reccon2(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.ep != tp && a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| filter(c, duel, tp), 1, None))
}

fn rectg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(600);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 600);
    Ok(false)
}

fn recop2(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
