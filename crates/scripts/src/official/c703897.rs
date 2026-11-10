// Orcust Crescendo
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, REMOVED, SZONE};
use engine::constants::MSG_CHAIN_NEGATED;

const ID: u32 = 703897;

// Galatea on the field to enable the negation, a copy in the Graveyard, and Girsu (a DARK Machine) in the Deck and banished to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (30741503, MZONE),
    (30741503, MZONE),
    (ID, GRAVE),
    (ID, SZONE),
    (69811710, DECK),
    (69811710, DECK),
    (69811710, REMOVED),
];
// Neither effect has a description, so ResolvesEffect(0) covers both; a chain link negated is not necessarily the card's own.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_CHAIN_NEGATED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(|duel, c| sumfilter(c, duel)));
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ORCUST]) && c.is_link_monster(duel)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_chain_negatable(a.ev as u8)
        && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(
            rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || (!relation && duel.is_player_can_remove(tp, None))
        );
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if relation {
        duel.set_operation_info(
            0,
            CATEGORY_REMOVE,
            Some(rc.into()),
            1,
            rc.get_controler(duel),
            rc.get_location(duel) as i32,
        );
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, 0, rc.get_previous_location(duel) as i32);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_DARK)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(Cost::self_banish(duel, a, 0)? && duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    Cost::self_banish(duel, a, 1)?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !sumfilter(c, duel));
    duel.register_effect(e1, tp);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel)) && sumfilter(c, duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_REMOVED, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK | LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
