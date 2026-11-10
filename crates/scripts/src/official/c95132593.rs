// Hecahands Gaigas
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 95132593;

// In the hand and on the field, with the opponent's Pot of Greed and Upstart Goblin to draw from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (55144522, HAND | OPPONENT), (70368879, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

// The "neither can be destroyed by battle" effect all Hecahands monsters share.
pub fn add_indestructable_battle(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| {
        let handler = e.get_handler(duel);
        c == handler || Some(c) == handler.get_battle_target(duel)
    });
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}

// The self Special Summon of Hecahands Gaigas and Breus, when a card goes to the opponent's hand outside the Draw Phase.
pub fn add_self_special_summon(duel: &mut Duel, c: CardId, id: u32) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(id, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, id);
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_controler(duel, 1 - a.tp), 1, None)
            && !duel.is_phase(PHASE_DRAW)
    });
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_indestructable_battle(duel, c);
    add_self_special_summon(duel, c, ID);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_DECK) >= 3);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, 0, LOCATION_DECK) == 0 {
        return Ok(());
    }
    duel.confirm_decktop(1 - tp, 3)?;
    let g = duel.get_decktop_group(1 - tp, 3);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && g.is_exists(duel, |c, duel| spfilter(c, duel, e, tp), 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g
            .filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp), 1, 1, None)?
            .get_first()
        {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.shuffle_deck(1 - tp)?;
    Ok(())
}
