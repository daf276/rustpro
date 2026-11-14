// Trito Enneacraft - "exapatisIA"
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 44716748;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone, face-down on the field and in the Deck to reveal and search, with the opponent's Set Dark
// Holes to destroy and copies in the hand, whose face-down summon it answers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
    (ID, HAND | OPPONENT),
    (ID, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::pay_lp::<900>);
    e2.set_target(duel, rvtg);
    e2.set_operation(duel, rvop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_FLIP);
    e3.set_target(duel, fliptg);
    e3.set_operation(duel, flipop);
    c.register_effect(duel, e3);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_facedown(duel) && a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_has_category(duel, CATEGORY_SET))
    });
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    register_set_summon::<ID>(duel, c);
}

// The Pendulum procedure and the E.C. Counter placed on each flip, which every Enneacraft Pendulum Monster opens with.
pub fn register_counter(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit_full(duel, COUNTER_EC, LOCATION_PZONE as u16);
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_FLIP);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_operation(duel, counterop);
    c.register_effect(duel, e1);
}

fn counterop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !duel.is_chain_solving() {
        c.add_counter(duel, COUNTER_EC, 1);
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).add_counter(duel, COUNTER_EC, 1);
        Ok(())
    });
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_label_object(duel, Some(e1));
    e2.set_operation(duel, |duel, a| {
        if let Some(e1) = a.e.get_label_object(duel) {
            e1.reset(duel);
        }
        Ok(())
    });
    e2.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e2, a.tp);
    Ok(())
}

fn rvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_able_to_hand(duel) && !c.is_public(duel)
}

fn rvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rvfilter, a.tp, LOCATION_DECK, 0, 3, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, rvfilter, tp, LOCATION_DECK, 0, 3, 3, None)?;
    if g.len() != 3 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let sg = g.random_select(duel, 1 - tp, 1);
    duel.shuffle_deck(tp)?;
    if !sg.is_empty() {
        duel.disable_shuffle_check(true);
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn fliptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - a.tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn flipop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(tp, |c, duel| c.is_spell_trap(duel), tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if dg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(dg);
    if duel.destroy(dg, REASON_EFFECT)? > 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        let e = a.e;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| set_filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

// The Quick Effect a face-down Enneacraft monster activates by flipping itself face-up; the card sets what it does.
pub fn flip_quick<const ID: u32>(duel: &mut Duel, c: CardId, category: u64) -> EffectId {
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, category);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, Cost::self_change_position::<POS_FACEUP_DEFENSE>);
    e4
}

// The Special Summon from the hand in face-down Defense Position of the Enneacraft monsters that list it fourth.
pub fn register_set_summon<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_HAND);
    e5.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::self_reveal, spcost::<ID>]));
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_position(duel, POS_FACEDOWN_DEFENSE)),
    );
}

fn spcost<const ID: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_FORCE_SPSUMMON_POSITION);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, POS_FACEDOWN_DEFENSE as i32);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn set_filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| set_filter(c, duel, e, tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| set_filter(c, duel, e, tp), LOCATION_HAND, POS_FACEDOWN_DEFENSE)
}
