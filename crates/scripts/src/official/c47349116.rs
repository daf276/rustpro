// Timestar Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, PZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 47349116;

// In the Extra Deck, Xyz Summoned with Level 4 Pendulum Monsters of the "Magician" archetype (White Wing Magician),
// with more on the field to protect from the opponent's Raigeki, and DARK Spellcasters (Tuning Magician, Dark Magician) to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (11067666, MZONE),
    (11067666, MZONE),
    (11067666, MZONE),
    (11067666, MZONE),
    (11067666, MZONE),
    (54941203, DECK),
    (54941203, DECK),
    (36996508, GRAVE),
    (11067666, PZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(matfilter);
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MUSTBEMAT: bool = true;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(
        duel,
        Value::SpSummonCondition(|duel, e, se, _, st, _, _| {
            !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || (st & SUMMON_TYPE_XYZ == SUMMON_TYPE_XYZ && se.is_none())
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, reptg);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_MAGICIAN], Some(xyz), sumtype, tp) && c.is_type_full(duel, TYPE_PENDULUM, Some(xyz), sumtype, tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_EXTRA))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_DECK | LOCATION_GRAVE | LOCATION_EXTRA;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, location as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, tc)?;
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE | LOCATION_PZONE)
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_grave(duel)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    if duel.select_effect_yes_no_full(tp, a.e.get_handler(duel), 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let sg = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        duel.hint(HINT_CARD, 0, ID as u64);
        duel.send_to_grave(sg, REASON_EFFECT)?;
        return Ok(true);
    }
    Ok(false)
}
