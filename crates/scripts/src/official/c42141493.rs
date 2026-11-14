// Mulcharmy Fuwalos
// Mulcharmy Purulia shares the functions generic over the card's id.
use crate::prelude::*;

const ID: u32 = 42141493;

pub const LISTED_SERIES: &[u16] = &[SET_MULCHARMY];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_ONFIELD, 0) == 0);
    e1.set_cost(duel, effcost::<ID>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    add_mulcharmy_counter(duel, ID);
}

pub(crate) fn add_mulcharmy_counter(duel: &mut Duel, id: u32) {
    let filter =
        |duel: &Duel, re: EffectId, _, _| !(re.get_handler(duel).is_set_card(duel, &[SET_MULCHARMY]) && re.is_monster_effect(duel));
    duel.add_custom_activity_counter(id, ACTIVITY_CHAIN, CounterFilter::Chain(filter));
}

pub(crate) fn effcost<const ID: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_discardable(duel) && duel.get_custom_activity_count(ID, tp, ACTIVITY_CHAIN) < 2);
    }
    duel.send_to_grave(c, REASON_COST | REASON_DISCARD)?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        duel.get_custom_activity_count(ID, a.e.get_handler_player(duel), ACTIVITY_CHAIN) >= 2
    });
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_handler(duel).is_set_card(duel, &[SET_MULCHARMY]) && re.is_monster_effect(duel)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

pub(crate) fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, drcon);
    e1.set_operation(duel, drop::<ID>);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    register_shuffle::<ID>(duel, c, a.tp);
    Ok(())
}

// The End Phase effect that shuffles random cards from the hand into the Deck.
pub(crate) fn register_shuffle<const ID: u32>(duel: &mut Duel, c: CardId, tp: u8) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_operation(duel, tdop::<ID>);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
}

fn drconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, 1 - tp) && c.is_summon_location(duel, LOCATION_DECK | LOCATION_EXTRA)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| drconfilter(c, duel, a.tp), 1, None)
}

// e2's closure captures e and e1: e1 is its label object, and e is e1's.
pub(crate) fn drop<const ID: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_chain_solving() {
        duel.hint(HINT_CARD, 1 - tp, u64::from(ID));
        duel.draw(tp, 1, REASON_EFFECT)?;
        return Ok(());
    }
    if let Some(eff) = e.get_label_object(duel).filter(|eff| !eff.is_deleted(duel)) {
        eff.set_label(duel, eff.get_label(duel) + 1);
        return Ok(());
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_operation(duel, chainsolvedop::<ID>);
    e1.set_label(duel, 1);
    e1.set_label_object(duel, Some(e));
    e1.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e1, tp);
    e.set_label_object(duel, Some(e1));
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_label_object(duel, Some(e1));
    e2.set_operation(duel, |duel, a| {
        let Some(e1) = a.e.get_label_object(duel) else { return Ok(()) };
        if let Some(e) = e1.get_label_object(duel) {
            e.set_label_object(duel, None);
        }
        e1.reset(duel);
        Ok(())
    });
    e2.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e2, tp);
    Ok(())
}

fn chainsolvedop<const ID: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_CARD, 1 - tp, u64::from(ID));
    duel.draw(tp, e.get_label(duel) as u32, REASON_EFFECT)?;
    e.reset(duel);
    if let Some(owner) = e.get_label_object(duel) {
        owner.set_label_object(duel, None);
    }
    Ok(())
}

fn tdop<const ID: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let dif = duel.get_field_group_count(tp, LOCATION_HAND, 0) as i64 - (duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) as i64 + 6);
    if dif > 0 {
        let hand = duel
            .get_field_group(tp, LOCATION_HAND, 0)
            .filter(duel, |c, duel| c.is_able_to_deck(duel), None);
        let g = hand.random_select(duel, tp, dif as usize);
        if g.is_empty() {
            return Ok(());
        }
        duel.hint(HINT_CARD, 1 - tp, u64::from(ID));
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
