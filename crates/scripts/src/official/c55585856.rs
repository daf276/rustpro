// Misstic Call
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND};

const ID: u32 = 55585856;

// The delayed effect stands in for the effect the Lua closure captures in the Special Summon check.
struct AddOrSummon;

impl aux::DelayedOperation for AddOrSummon {
    fn operate(duel: &mut Duel, ag: CardSet, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let Some(tc) = ag.get_first() else { return Ok(()) };
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |tc, duel| duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| {
                duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
                Ok(())
            },
            aux::stringid(ID, 2),
        )
    }
}

// In the hand and Graveyard, with Pot of Greed to discard and Aqua Madoor (Level 4 Spellcaster) in the Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (55144522, HAND),
    (85639257, DECK),
    (85639257, DECK),
    (85639257, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(duel, a, chk, selfspcostfilter, true, (1, 1))
    });
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_target(duel, bantg);
    e2a.set_operation(duel, banop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) || c.is_spell_trap(duel)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn banfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4]) && c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_remove(duel)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(banfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_REMOVED as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_REMOVED as i32);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = duel.select_matching_card(tp, banfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if rg.is_empty()
        || duel.remove(rg, POS_FACEUP, REASON_EFFECT)? == 0
        || !rg.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_REMOVED))
    {
        return Ok(());
    }
    aux::delayed_operation::<AddOrSummon>(duel, rg, PHASE_END, ID, e, tp);
    Ok(())
}
