// Plundered Power Patron Plane - Vidolia
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 51669847;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON];

const NERVA: u32 = 53589300;
const JUPITER: u32 = 68231287;
const JUNORA: u32 = 5914858;

pub const LISTED_NAMES: &[u32] = &[NERVA, JUPITER, JUNORA];

// Junora and Jupiter in the Extra Deck to reveal, and Elfnote Power Patron and Power Patron DoomZ in the Deck, which
// mention them. Nerva, Jupiter and Junora on the field for the Graveyard lock.
pub const SUPPORT: &[(u32, u16)] = &[
    (JUNORA, EXTRA),
    (JUPITER, EXTRA),
    (12375297, DECK),
    (81650695, DECK),
    (12375297, DECK),
    (81650695, DECK),
    (NERVA, MZONE),
    (JUPITER, MZONE),
    (JUNORA, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, revealcost);
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, _| -100 * duel.get_field_group_count(0, LOCATION_REMOVED, LOCATION_REMOVED) as i32),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, 0, 1);
    e3.set_condition(duel, cannotactcon);
    e3.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_activate_location(duel) == LOCATION_GRAVE),
    );
    c.register_effect(duel, e3);
}

fn cannotactcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    [NERVA, JUPITER, JUNORA].into_iter().all(|code| {
        duel.is_existing_matching_card(
            aux::faceup_filter(move |c, duel| c.is_code(duel, code)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    })
}

fn revealfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON])
        && !c.is_public(duel)
        && duel.is_existing_matching_card(
            |sc, duel| thspfilter(sc, duel, e, tp, c.get_code(duel), duel.get_location_count(tp, LOCATION_MZONE) > 0),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let record: fn(&mut Duel, &Args, CardSet) = |duel, a, g| {
        if let Some(rc) = g.get_first() {
            a.e.set_label(duel, i64::from(rc.get_code(duel)));
        }
    };
    Cost::reveal(duel, a, chk, revealfilter, false, (1, 1), Some(record), LOCATION_EXTRA)
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32, mmz_chk: bool) -> bool {
    c.lists_code(duel, code)
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn removable(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove_full(duel, None, POS_FACEDOWN, REASON_EFFECT)
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(removable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, removable, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if g.is_empty() || duel.remove(g, POS_FACEDOWN, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let code = e.get_label(duel) as u32;
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let sg = duel.select_matching_card(
        tp,
        |c, duel| thspfilter(c, duel, e, tp, code, mmz_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = sg.get_first() else { return Ok(()) };
    duel.break_effect()?;
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| mmz_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 2),
    )
}
