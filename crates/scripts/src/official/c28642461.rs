// K9-66a Jokul
use super::c55031170::limit_xyz_material;
use super::c92248362::ntcon;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 28642461;

// In the hand with K9-66b Lantern, a Level 5 monster, to reveal and Special Summon with, and K9-17 Izuna in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (55031170, HAND), (92248362, DECK), (92248362, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[5]) && !c.is_public(duel) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_public(duel)
            && duel.is_existing_matching_card(|sc, duel| spcostfilter(sc, duel, e, tp), tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| spcostfilter(sc, duel, e, tp),
        tp,
        LOCATION_HAND,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    let rc = g.get_first();
    duel.confirm_cards(1 - tp, CardSet::from(c).union(g))?;
    e.set_label_object_group(duel, rc.map(CardSet::from));
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff));
    }
    let rc = e.get_label_object_group(duel).unwrap_or_default();
    duel.set_target_card(rc);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(CardSet::from(c).union(rc)), 2, tp, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = CardSet::from(c).union(duel.get_first_target().map(CardSet::from).unwrap_or_default());
    if g.filter_count(duel, |sc, duel| spfilter(sc, duel, e, tp), None) == 2
        && duel.get_location_count(tp, LOCATION_MZONE) >= 2
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 2
    {
        limit_xyz_material(duel, c, g, aux::stringid(ID, 3));
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_race(duel, RACE_AQUA) && c.is_set_card(duel, &[SET_K9]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
