// Ningirsu the World Chalice Warrior
use super::c31226177::{spcon2, spop2, sptg2};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 30194529;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_CHALICE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Link Spiders (Link-1), with Lee the World Chalice Fairy in the hand to Special Summon,
// the opponent's Sangans to send to the Graveyard, and Raigeki from the opponent's hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (98978921, MZONE),
    (98978921, MZONE),
    (98978921, MZONE),
    (21893603, HAND),
    (21893603, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_condition(duel, spcon2);
    e5.set_target(duel, sptg2);
    e5.set_operation(duel, spop2);
    c.register_effect(duel, e5);
}

fn linked_world_chalice_count(duel: &Duel, c: CardId) -> usize {
    c.get_linked_group(duel)
        .filter_count(duel, aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_WORLD_CHALICE])), None)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let gc = linked_world_chalice_count(duel, a.e.get_handler(duel));
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, gc as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let gc = linked_world_chalice_count(duel, a.e.get_handler(duel));
    if gc > 0 {
        duel.draw(p, gc as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) > 0 && duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) == 0 || duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g1 = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g2 = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.send_to_grave(g1.union(g2), REASON_EFFECT)?;
    Ok(())
}
