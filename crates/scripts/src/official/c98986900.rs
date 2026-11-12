// Thelematech Clatis
use engine::constants::MSG_SELECT_EFFECTYN;

use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 98986900;

// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
// On the field and Xyz Summoned with Level 8 Normal Monsters (Black Luster Soldier), with Raigeki in hand to send the opponent's
// Sangans to the Graveyard for their effects to chain, Dark Hole in the opponent's hand to destroy it, and Spellcasters in the Deck to
// search and to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (10000100, MZONE),
    (12580477, HAND),
    (12580477, HAND),
    (53129443, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (11067666, DECK),
    (11067666, DECK),
    (17985575, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_SPELL);
    c.set_counter_limit(duel, COUNTER_SPELL, 9);
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<8, 2>>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_CHAINING);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, acop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, ctcost);
    e2.set_target(duel, cttg);
    e2.set_operation(duel, ctop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, reptg);
    c.register_effect(duel, e3);
}

fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.ep == 1 - a.tp && a.re.is_some_and(|re| !re.is_spell_effect(duel)) && c.has_flag_effect(duel, 1) {
        c.add_counter(duel, COUNTER_SPELL, 1);
    }
    Ok(())
}

fn ctcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_remove_counter(duel, a.tp, COUNTER_SPELL, 3, REASON_COST));
    }
    c.remove_counter(duel, a.tp, COUNTER_SPELL, 3, REASON_COST)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_spell(duel) || (c.is_race(duel, RACE_SPELLCASTER) && c.is_type(duel, TYPE_EFFECT))) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
            || (duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let thg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    let spg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    let b1 = !thg.is_empty();
    let b2 = !spg.is_empty() && duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = thg.select(duel, tp, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    } else if op == Some(2) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = spg.select(duel, tp, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT | REASON_REPLACE)?;
    Ok(true)
}
