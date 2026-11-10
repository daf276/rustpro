// Magicians' Souls
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 97631303;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL];
// Two copies (hand and field), with Dark Magician, a Level 7 Spellcaster Normal Monster, in the Deck to send and in the Graveyard to
// Summon, and Mystical Space Typhoon in the hand and set to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, GRAVE),
    (5318639, HAND),
    (5318639, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, drcost);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_level_above(duel, 6) && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let b2 = c.is_able_to_grave(duel);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel
        .select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
        .unwrap_or(0);
    e.set_label(duel, op as i64);
    if op == 1 {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    } else if op == 2 {
        e.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(c.into()), 1, 0, 0);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_DARK_MAGICIAN) || c.is_code(duel, CARD_DARK_MAGICIAN_GIRL))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let (op, c) = (e.get_label(duel), e.get_handler(duel));
    if op == 1 && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    } else if op == 2
        && c.is_relate_to_effect(duel, e)
        && duel.send_to_grave(c, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_GRAVE)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_GRAVE,
            0,
            None,
        );
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 4))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn drcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(drcfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, 1, None));
    }
    let ft = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    let g = duel.get_matching_group(drcfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, None);
    let ct = ft.min(g.len());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sg = g.select(duel, tp, 1, ct.min(2) as u8, None)?;
    a.e.set_label(duel, sg.len() as i64);
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    let label = e.get_label(duel);
    duel.set_target_player(tp);
    duel.set_target_param(label as u32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, label as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
