// Crystron Sulfefnir
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 3422200;
const CRYSTRON_SULFADOR: u32 = 25865565;
const CRYSTRON_SMIGER: u32 = 83443619;
const DARK_HOLE: u32 = 53129443;

pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand, on the field and in the Graveyard, other Crystron monsters in the hand to discard and in the Deck to
// summon, and the opponent's Dark Hole to destroy the copy on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (CRYSTRON_SMIGER, HAND),
    (CRYSTRON_SMIGER, DECK),
    (CRYSTRON_SULFADOR, DECK),
    (DARK_HOLE, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(
            duel,
            a,
            chk,
            |c, duel| c.is_set_card(duel, &[SET_CRYSTRON]) && !c.is_code(duel, ID),
            true,
            (1, 1),
        )
    });
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.r & (REASON_BATTLE | REASON_EFFECT) > 0 && a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD)
    });
    e2.set_target(duel, decksptg);
    e2.set_operation(duel, deckspop);
    c.register_effect(duel, e2);
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, a.e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, 0);
    if !g.is_empty() {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_CRYSTRON]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn decksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| deckspfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn deckspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| deckspfilter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        POS_FACEUP_DEFENSE,
    )
}
