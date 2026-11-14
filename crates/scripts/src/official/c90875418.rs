// Theorealize Medius
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 90875418;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON, SET_THEOREALIZE, SET_ARS_MAGNA, SET_DIACTORUS];

const CHOICE: u8 = 0;

// Artmage Power Patron as the "Power Patron" card on the field, Theorealize cards in the Deck to add, an Ars Magna monster in hand to
// banish and others in the Deck to add, and Artmage Diactorus in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (23829452, MZONE),
    (27285068, DECK),
    (36709484, DECK),
    (36270527, HAND),
    (36270527, HAND),
    (62368221, DECK),
    (99753860, DECK),
    (27184601, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(27184601),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, handspcon);
    e1.set_target(duel, handsptg);
    e1.set_operation(duel, handspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, effcon);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn handspcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_POWER_PATRON])),
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        None,
    )
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tohandtheorealizefilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THEOREALIZE]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(tohandtheorealizefilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, tohandtheorealizefilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn effconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_faceup(duel) && c.is_location(duel, LOCATION_REMOVED)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, effconfilter, 1, None)
}

fn tohandarsmagnafilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARS_MAGNA]) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: Option<CardId>) -> bool {
    c.is_set_card(duel, &[SET_DIACTORUS])
        && duel.get_location_count_from_ex(tp, tp, mc.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let option_1 = duel.is_existing_matching_card(tohandarsmagnafilter, tp, LOCATION_DECK, 0, 1, None);
    let option_2 = c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, Some(c)), tp, LOCATION_EXTRA, 0, 1, None);
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 3)), (option_2, aux::stringid(ID, 4))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        e.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let choice = e.get_chain_data(duel, CHOICE).unwrap_or(0);
    if choice == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, tohandarsmagnafilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if choice == 2 {
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) && duel.remove(c, POS_FACEUP, REASON_EFFECT)? > 0 && c.is_location(duel, LOCATION_REMOVED) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}
