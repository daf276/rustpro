// Ursarctic Polari
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 27693363;

pub const LISTED_NAMES: &[u32] = &[CARD_URSARCTIC_BIG_DIPPER];

// In the Extra Deck, Special Summoned by sending Gusto Egul (a Level 1 Tuner) and Gusto Griffin (Level 2) to the Graveyard, with Ursarctic
// Big Dipper in the Deck to activate, and on the field with Ursarctic Megabilis in the hand to Tribute and one in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (91662792, MZONE),
    (42082363, MZONE),
    (42082363, MZONE),
    (CARD_URSARCTIC_BIG_DIPPER, DECK),
    (CARD_URSARCTIC_BIG_DIPPER, DECK),
    (81108658, HAND),
    (81108658, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, sprcon);
    e0.set_target(duel, sprtg);
    e0.set_operation(duel, sprop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, acttg);
    e1.set_operation(duel, actop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable_full(duel, a, chk, thspcost, extracon));
    e2.set_target(duel, thsptg);
    e2.set_operation(duel, thspop);
    c.register_effect(duel, e2);
}

fn sprfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel) && c.has_level(duel)
}

fn sprfilter1(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    c.is_type(duel, TYPE_TUNER) && g.is_exists(duel, |gc, duel| sprfilter2(gc, duel, tp, c, sc), 1, Some(c.into()))
}

fn sprfilter2(c: CardId, duel: &Duel, tp: u8, mc: CardId, sc: CardId) -> bool {
    let sg = CardSet::from_iter([c, mc]);
    c.get_level(duel).abs_diff(mc.get_level(duel)) == 1
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
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    let g1 = g.filter(duel, |gc, duel| sprfilter1(gc, duel, tp, c), None);
    let mut mg1 = aux::select_unselect_group_full(duel, g1, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TOGRAVE, None, true)?;
    if let Some(mc) = mg1.get_first() {
        let g2 = g.filter(duel, |gc, duel| sprfilter2(gc, duel, tp, mc, c), Some(mc.into()));
        let mg2 = aux::select_unselect_group_full(duel, g2, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TOGRAVE, None, true)?;
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

fn actfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CARD_URSARCTIC_BIG_DIPPER)
        && c.get_activate_effect(duel)
            .is_some_and(|te| te.is_activatable(duel, tp, true, true))
}

fn acttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| actfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, |c, duel| actfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.activate_field_spell(sc, tp)?;
    }
    Ok(())
}

fn thspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 7) && {
        let mmz_chk = duel.get_mzone_count(tp, Some(c.into())) > 0;
        duel.is_existing_matching_card(|gc, duel| thspfilter(gc, duel, e, tp, mmz_chk), tp, LOCATION_GRAVE, 0, 1, None)
    }
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_URSARCTIC])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| thspcostfilter(c, duel, e, tp), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| thspcostfilter(c, duel, e, tp), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, |c, duel| thspfilter(c, duel, e, tp, mmz_chk), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| mmz_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 3),
    )
}

fn extracon(duel: &Duel, _: EffectId, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.is_existing_matching_card(
        |c, duel| thspfilter(c, duel, e, tp, mmz_chk),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        a.c.map(Into::into),
    )
}
