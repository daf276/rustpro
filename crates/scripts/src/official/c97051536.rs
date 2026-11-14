// Infernoid Evil
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 97051536;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID, SET_VOID];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_INFERNOID], Some(fc), sumtype, tp)); 2];
}

// Fusion Summoned by Polymerization from Infernoid Decatron and Pirmais, with Infernoids of each name in the Deck to send, Pirmais
// in the Graveyard to banish, the Void Spells in the Deck to search, and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (58446973, HAND),
    (60551528, HAND),
    (58446973, MZONE),
    (58446973, DECK),
    (60551528, DECK),
    (14799437, DECK),
    (23440231, DECK),
    (60551528, GRAVE),
    (61345801, DECK),
    (93946239, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_cost(duel, tgcost);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn cfilter(c: CardId, duel: &Duel, ct: usize) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID])
        && c.is_monster(duel)
        && c.has_level(duel)
        && c.is_level_below(duel, ct as u32)
        && c.is_able_to_remove_as_cost(duel)
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ct = duel
        .get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, ct), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, ct), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(rc) = g.get_first() else { return Ok(false) };
    duel.remove(rc, POS_FACEUP, REASON_COST)?;
    let level = rc.get_level(duel);
    a.e.set_label(duel, i64::from(level));
    Ok(false)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ct = a.e.get_label(duel);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, ct as u8, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = a.e.get_label(duel) as usize;
    let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, ct, ct, &aux::dncheck, tp, HINTMSG_TOGRAVE)?;
    if !sg.is_empty() {
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VOID]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
