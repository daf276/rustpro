// Celtic Mystic
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 50073633;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];
// A copy in the hand and on the field, with Magician of Dark Chaos - Black Chaos and Black Luster Soldier - Soldier of Light and
// Darkness (Ritual Monsters that mention Ritual of Light and Darkness) in the hand to reveal and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (44001993, HAND), (70405001, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, drtg);
    e1a.set_operation(duel, drop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(!duel.is_existing_matching_card(CardId::is_public, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 3);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, tp, 2);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, sprestrictioncon);
    e1.set_operation(duel, sprestrictionop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if duel.is_existing_matching_card(CardId::is_public, tp, LOCATION_HAND, 0, 1, None) {
        return Ok(());
    }
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    if g.is_exists(duel, |c, duel| c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL), 1, None)
        && duel.is_player_can_draw(tp, 3)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        && duel.draw(tp, 3, REASON_EFFECT)? == 3
    {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 2, 2, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn sprestrictioncon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| {
        eg.is_exists(
            duel,
            |c, duel| c.is_summon_player(duel, tp) && c.is_summon_location(duel, LOCATION_EXTRA),
            1,
            None,
        )
    })
}

fn sprestrictionop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check(duel, c, tp);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_ritual_monster(duel)
        && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, true, POS_FACEUP)?;
    }
    Ok(())
}
