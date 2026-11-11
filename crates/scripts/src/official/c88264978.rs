// Red-Eyes Darkness Metal Dragon (the script in pre-errata)
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 88264978;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand, Special Summoned by banishing Red-Eyes Black Dragon from the field, and on the field, with Dragons in the
// Graveyard and hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_REDEYES_B_DRAGON, MZONE),
    (CARD_REDEYES_B_DRAGON, GRAVE),
    (CARD_REDEYES_B_DRAGON, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon1);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_remove_as_cost(duel) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let rg = duel.get_matching_group(|c, duel| spfilter(c, duel, ft), tp, LOCATION_MZONE, 0, None);
    ft > -1 && !rg.is_empty() && aux::select_unselect_group_check(duel, rg, 1, 1, &|_, _, _, _| (true, false))
}

fn sptg1(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let rg = duel.get_matching_group(|c, duel| spfilter(c, duel, ft), tp, LOCATION_MZONE, 0, None);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_GRAVE | LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| filter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
