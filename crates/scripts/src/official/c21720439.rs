// Artmage Movement -Pedigree-
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 21720439;
const ACROPOLIS: u32 = 74733322;

// In hand to activate against monsters of the opponent, or to discard as the cost of Acropolis (in hand to activate), with Artmage
// monsters in the Deck to Special Summon and Artmage Impasto -Recapture- in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ACROPOLIS, HAND),
    (ACROPOLIS, 0),
    (ACROPOLIS, 0),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (97434754, DECK),
    (34541940, DECK),
    (44654994, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > duel.get_field_group_count(a.tp, LOCATION_MZONE, 0)
    });
    e1.set_cost(duel, cost::<ID>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_cost(duel, cost::<ID>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

pub fn counterfilter(duel: &Duel, c: CardId) -> bool {
    !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_type(duel, TYPE_FUSION)
}

// Shared with Artmage Peripeteia -Turmoil-, whose counter is its own.
pub fn cost<const ID: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_FUSION)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

pub fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_reason(duel, REASON_COST)
        && a.re.is_some_and(|re| re.is_activated(duel))
        && duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_CODE) == ACROPOLIS
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, codes: &[i64]) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && !codes.contains(&i64::from(c.get_code(duel)))
}

fn fieldcodes(duel: &Duel, tp: u8) -> Vec<i64> {
    duel.get_matching_group(CardId::is_faceup, tp, LOCATION_ONFIELD, 0, None)
        .get_class(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
            return Ok(false);
        }
        let codes = fieldcodes(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, &codes), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        let codes = fieldcodes(duel, tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, &codes), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE]) && c.is_trap(duel) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE | LOCATION_REMOVED, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
