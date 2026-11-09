// First Penguin
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 12975671;

// In the hand and on the field, with a Synchro Monster to reveal in the Extra Deck, Brionac, the Magical Ice Dragon, and a Normal
// Monster one Level lower in the Deck, Spike Seadra.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (65892310, EXTRA), (85326399, DECK)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, selfspcon);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::discard, revealcost]));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(_) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && !duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_ADD_TYPE);
    e1.set_value(duel, TYPE_TUNER as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    Ok(())
}

fn revealfilter(c: CardId, duel: &Duel, _: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER)
        && c.is_synchro_monster(duel)
        && !c.is_level(duel, &[1])
        && !c.is_public(duel)
        && duel.is_existing_matching_card(
            |sc, duel| thfilter(sc, duel, c.get_race(duel), c.get_level(duel).saturating_sub(1)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let record: fn(&mut Duel, &Args, CardSet) = |duel, a, g| {
        if let Some(sc) = g.get_first() {
            a.e.set_labels(duel, &[sc.get_race(duel) as i64, i64::from(sc.get_level(duel)) - 1]);
        }
    };
    Cost::reveal(duel, a, chk, revealfilter, false, (1, 1), Some(record), LOCATION_EXTRA)
}

fn thfilter(c: CardId, duel: &Duel, race: u64, lv: u32) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_race(duel, race) && c.is_level(duel, &[lv]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(
        0,
        CATEGORY_POSITION,
        Some(a.e.get_handler(duel).into()),
        1,
        tp,
        POS_FACEDOWN_DEFENSE as i32,
    );
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let &[race, lv] = e.get_labels(duel) else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| thfilter(sc, duel, race as u64, lv as u32),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first()
        && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        if c.is_relate_to_effect(duel, e) && c.is_can_turn_set(duel) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.change_position(c, POS_FACEDOWN_DEFENSE)?;
        }
    }
    limit_extra_deck_to_water(duel, c, tp, aux::stringid(ID, 3));
    Ok(())
}

// The restriction Abyssrhine, the Atlantean Spirit also applies: only WATER monsters from the Extra Deck for the rest of the turn.
pub fn limit_extra_deck_to_water(duel: &mut Duel, c: CardId, tp: u8, description: u64) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, description);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_attribute(duel, ATTRIBUTE_WATER)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_attribute(duel, ATTRIBUTE_WATER));
}
