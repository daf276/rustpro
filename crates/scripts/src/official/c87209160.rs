// Tri-Brigade Fraktall
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 87209160;

// In the hand and on the field, with Tri-Brigade Shuraig in the Extra Deck, four Beast monsters in the Graveyard to banish
// for its Link Rating and more in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (99726621, EXTRA),
    (6297941, GRAVE),
    (6297941, GRAVE),
    (6297941, GRAVE),
    (7805359, GRAVE),
    (6297941, DECK),
    (7805359, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(99726621),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop::<ID>);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_level_below(duel, 3) && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ct: u32, g: Option<CardSet>) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && c.is_type(duel, TYPE_LINK)
        && c.is_link(duel, &[ct])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
        && duel.get_location_count_from_ex(tp, tp, g, c) > 0
}

pub fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let nums: Vec<u64> = (1..=g.len() as u32)
        .filter(|&i| duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, i, Some(g)), tp, LOCATION_EXTRA, 0, 1, None))
        .map(u64::from)
        .collect();
    if chk == 0 {
        return Ok(!nums.is_empty());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
    let ct = duel.announce_number(tp, &nums)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = g.select(duel, tp, ct as u8, ct as u8, None)?;
    duel.remove(rg, POS_FACEUP, REASON_COST)?;
    e.set_label(duel, ct as i64);
    Ok(false)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_EXTRA as i32);
    Ok(false)
}

pub fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB));
    e1.set_target_range(duel, LOCATION_ALL, LOCATION_ALL);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(c.is_controler(duel, e.get_handler_player(duel)))),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(CODE, 2)));
    let ct = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, ct, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
