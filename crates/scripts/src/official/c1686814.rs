// Ultimaya Tzolkin
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 1686814;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_TOOL];
// In the Extra Deck, with a Level 5 Tuner (Ally Mind) and a Level 5 non-Tuner (Zanki) to send to the Graveyard,
// Level 7 and 8 Dragon Synchro Monsters (Exploder Dragonwing, Light End Dragon) to summon, and Trap Hole in hand to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (40155554, MZONE),
    (40155554, MZONE),
    (30090452, MZONE),
    (30090452, MZONE),
    (40529384, EXTRA),
    (25132288, EXTRA),
    (4206964, HAND),
    (4206964, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

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
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_condition(duel, sprcon);
    e2.set_target(duel, sprtg);
    e2.set_operation(duel, sprop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SSET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, tgcon);
    e4.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e5.set_value(duel, 1);
    c.register_effect(duel, e5);
}

fn sprfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_level(duel) > 4 && c.is_able_to_grave_as_cost(duel)
}

fn sprfilter1(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    let lv = c.get_level(duel);
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    c.is_type(duel, TYPE_TUNER) && g.is_exists(duel, |mc, duel| sprfilter2(mc, duel, tp, c, sc, lv), 1, Some(c.into()))
}

fn sprfilter2(c: CardId, duel: &Duel, tp: u8, mc: CardId, sc: CardId, lv: u32) -> bool {
    let sg = CardSet::from_iter([c, mc]);
    c.get_level(duel) == lv && !c.is_type(duel, TYPE_TUNER) && duel.get_location_count_from_ex(tp, tp, Some(sg), sc) > 0
}

fn sprcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    g.is_exists(duel, |mc, duel| sprfilter1(mc, duel, tp, c), 1, None)
}

fn sprtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE, 0, None);
    let g1 = g.filter(duel, |mc, duel| sprfilter1(mc, duel, tp, c), None);
    let mut mg1 = aux::select_unselect_group_full(duel, g1, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TOGRAVE, None, true)?;
    if let Some(mc) = mg1.get_first() {
        let lv = mc.get_level(duel);
        let g2 = g.filter(duel, |c2, duel| sprfilter2(c2, duel, tp, mc, c, lv), Some(mc.into()));
        let mg2 = aux::select_unselect_group_full(duel, g2, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TOGRAVE, None, true)?;
        mg1.merge(mg2);
    }
    if mg1.len() == 2 {
        e.set_label_object_group(duel, Some(mg1));
        return Ok(true);
    }
    Ok(false)
}

fn sprop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.send_to_grave(g, REASON_COST)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_POWER_TOOL]) || (c.is_level(duel, &[7, 8]) && c.is_race(duel, RACE_DRAGON)))
        && c.is_type(duel, TYPE_SYNCHRO)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let e = a.e;
    let tp = a.tp;
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, e, tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_SYNCHRO)),
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}
