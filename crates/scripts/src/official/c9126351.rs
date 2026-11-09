// Swap Frog
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 9126351;

// In the hand and on the field, with Psychic Kappa, a WATER Normal Monster, in the hand to discard and Turu-Purun, a WATER
// Level 2 Aqua Normal Monster, in the Deck to send. No other "Frog" is ported, so the extra Normal Summon has no beneficiary.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (7892180, HAND), (59053232, DECK), (59053232, DECK)];
// Both the send to the Graveyard and the extra Normal Summon have the description #1.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_cost(duel, excost);
    e5.set_target(duel, extg);
    e5.set_operation(duel, exop);
    c.register_effect(duel, e5);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_discardable(duel)
}

fn discard_group(duel: &Duel, e: EffectId, tp: u8) -> CardSet {
    duel.get_matching_group(spfilter, tp, LOCATION_HAND, 0, Some(e.get_handler(duel).into()))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    aux::select_unselect_group_check(duel, discard_group(duel, a.e, tp), 1, 1, &|_, _, _, _| (true, false))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = discard_group(duel, a.e, tp);
    let rg = aux::select_unselect_group_full(duel, g, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_DISCARD, None, true)?;
    if rg.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(rg));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(rg) = a.e.get_label_object_group(duel) else {
        return Ok(());
    };
    duel.send_to_grave(rg, REASON_COST | REASON_DISCARD)?;
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 2)
        && c.is_attribute(duel, ATTRIBUTE_WATER)
        && c.is_race(duel, RACE_AQUA)
        && (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel))
        && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, a.tp, LOCATION_DECK | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, (LOCATION_DECK | LOCATION_MZONE) as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK | LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn excost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_flag_effect(tp, ID) == 0
            && duel.is_existing_matching_card(CardId::is_able_to_hand_as_cost, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(tp, CardId::is_able_to_hand_as_cost, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    Ok(false)
}

fn extg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || (duel.is_player_can_summon(a.tp) && duel.is_player_can_additional_summon(a.tp)))
}

fn exop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_FROG]) && c.get_code(duel) != ID);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}
