// Black Chaos
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 98684220;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];
// A copy in the hand, Magician of Black Chaos to shuffle into the Deck, and Black Magic Ritual for the immunity.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (30208479, GRAVE), (76792184, GRAVE)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_ritual_spell, a.e.get_handler_player(duel), LOCATION_GRAVE, 0, 1, None)
    });
    e2.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner_player(duel) != e.get_handler_player(duel) && te.is_activated(duel)),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER | RACE_WARRIOR) && c.is_ritual_monster(duel) && c.is_able_to_deck_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(spconfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(spconfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, None);
    let sg = aux::select_unselect_group_full(duel, g, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TODECK, None, true)?;
    if sg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    if let Some(sc) = g.get_first() {
        if sc.is_location(duel, LOCATION_HAND) {
            duel.confirm_cards(1 - a.tp, sc)?;
        } else {
            duel.hint_selection(sc);
        }
    }
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
    Ok(())
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_continuous_trap(duel)
        && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL)
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = aux::necro_valley_filter(|c, duel| plfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 2, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, 2, 2, None)?;
    if g.len() == 2 {
        duel.hint_selection(g);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
