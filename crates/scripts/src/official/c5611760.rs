// Rainbow Bridge of Salvation
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, SZONE};

const ID: u32 = 5611760;

pub const LISTED_SERIES: &[u16] = &[SET_CRYSTAL_BEAST];
// Level 10 monsters of two races (Gem-Knight Lady Brilliant Diamond, Leo) to activate; a copy in the GY to search the
// Crystal Beasts and Umi in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (19355597, MZONE),
    (8561192, MZONE),
    (68215963, DECK),
    (21698716, DECK),
    (22702055, DECK),
    (22702055, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, tdcon);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID + 1) == 0);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

const LOCATIONS: u32 = LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE;

fn tdcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(
        |c, duel| c.is_faceup(duel) && c.is_level_above(duel, 10),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    g.len() > 1 && g.get_class_count(duel, |c, duel| c.get_race(duel) as i64) > 1 && duel.get_flag_effect(a.tp, ID) == 0
}

// The Lua passes the handler as IsAbleToDeck's extra argument rather than as the exception, so this card is included.
fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATIONS, LOCATIONS, None);
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_player_can_draw(tp, 5) && duel.is_player_can_draw(1 - tp, 5));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 5);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_flag_effect(tp, ID) != 0 {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, 0, 0, 0);
    let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATIONS, LOCATIONS, None);
    if !g.is_empty() && duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0 {
        let og = duel.get_operated_group();
        if !og.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), 1, None) {
            return Ok(());
        }
        let dg = og.filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
        let ct = dg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
        if ct > 0 {
            duel.shuffle_deck(tp)?;
        }
        if dg.len() > ct {
            duel.shuffle_deck(1 - tp)?;
        }
        duel.break_effect()?;
        duel.draw(tp, 5, REASON_EFFECT)?;
        duel.draw(1 - tp, 5, REASON_EFFECT)?;
    }
    Ok(())
}

fn cbfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_CRYSTAL_BEAST])
        && c.is_able_to_hand(duel)
        && duel.is_existing_matching_card(fsfilter, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn fsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FIELD) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cbfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_flag_effect(tp, ID + 1) != 0 {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID + 1, 0, 0, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| cbfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = g.union(duel.select_matching_card(tp, fsfilter, tp, LOCATION_DECK, 0, 1, 1, Some(g))?);
    if g.len() < 2 {
        return Ok(());
    }
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    Ok(())
}
