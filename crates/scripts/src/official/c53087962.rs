// Ursarctic Septentrion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 53087962;

// In the Extra Deck, Special Summoned by sending Ursarctic Megabilis (a Level 8 Tuner) and Ursarctic Polari (a Level 1 Synchro Monster)
// to the Graveyard, with Ursarctic Megabilis in the Deck to search and Monster Reborn in the opponent's hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (81108658, MZONE),
    (27693363, MZONE),
    (27693363, MZONE),
    (81108658, DECK),
    (81108658, DECK),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (15023985, GRAVE | OPPONENT),
    (15023985, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

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
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, distg);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_count_limit(duel, 1, ID);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, thcon);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn sprfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel) && c.has_level(duel)
}

fn sprfilter1(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    c.is_type(duel, TYPE_TUNER)
        && c.is_level_above(duel, 8)
        && g.is_exists(duel, |gc, duel| sprfilter2(gc, duel, tp, c, sc), 1, Some(c.into()))
}

fn sprfilter2(c: CardId, duel: &Duel, tp: u8, mc: CardId, sc: CardId) -> bool {
    let sg = CardSet::from_iter([c, mc]);
    c.get_level(duel).abs_diff(mc.get_level(duel)) == 7
        && c.is_type(duel, TYPE_SYNCHRO)
        && !c.is_type(duel, TYPE_TUNER)
        && duel.get_location_count_from_ex(tp, tp, Some(sg), sc) > 0
}

pub fn sprcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    g.is_exists(duel, |gc, duel| sprfilter1(gc, duel, tp, c), 1, None)
}

pub fn sprtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    let g1 = g.filter(duel, |gc, duel| sprfilter1(gc, duel, tp, c), None);
    let rescon = |_: &Duel, _: CardSet, _: CardSet, _: Option<CardId>| (true, false);
    let mut mg1 = aux::select_unselect_group_full(duel, g1, 1, 1, &rescon, tp, HINTMSG_TOGRAVE, None, true)?;
    if let Some(mc) = mg1.get_first() {
        let g2 = g.filter(duel, |gc, duel| sprfilter2(gc, duel, tp, mc, c), Some(mc.into()));
        let mg2 = aux::select_unselect_group_full(duel, g2, 1, 1, &rescon, tp, HINTMSG_TOGRAVE, None, true)?;
        mg1.merge(mg2);
    }
    if mg1.len() == 2 {
        e.set_label_object_group(duel, Some(mg1));
        return Ok(true);
    }
    Ok(false)
}

pub fn sprop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    Ok(())
}

fn distg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA) && !c.has_level(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_URSARCTIC]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
