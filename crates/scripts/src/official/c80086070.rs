// Ursarctic Grand Chariot
use super::c53087962::{sprcon, sprop, sprtg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 80086070;

pub const LISTED_SERIES: &[u16] = &[SET_URSARCTIC];
// In the Extra Deck, Special Summoned by sending Ursarctic Megabilis (a Level 8 Tuner) and Ursarctic Polari (a Level 1 Synchro Monster)
// to the Graveyard, with Mind Control in the opponent's hand to target an Ursarctic monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (81108658, MZONE),
    (27693363, MZONE),
    (27693363, MZONE),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_condition(duel, sprcon);
    e2.set_target(duel, sprtg);
    e2.set_operation(duel, sprop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_CARD_TARGET);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_NEGATE);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, discon);
    e4.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, discost));
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 2, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn disfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_location(duel, LOCATION_ONFIELD) && c.is_controler(duel, tp) && c.is_set_card(duel, &[SET_URSARCTIC])
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    if !a.re.is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(ev, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| disfilter(c, duel, tp), 1, None)) && duel.is_chain_negatable(ev)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (1, 1), true, None, None));
    }
    let sg = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), true, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_activation(a.ev as u8);
    Ok(())
}
