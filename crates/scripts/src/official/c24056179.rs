// Capsule Monster Chess
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 24056179;
const ORIGINAL_RACE: u8 = 0;
const ORIGINAL_ATTRIBUTE: u8 = 1;
const ORIGINAL_LEVEL: u8 = 2;

// In the hand to activate, with Baby Dragons (Level 3 WIND Dragon) on the field to send and in the Graveyard to place, and
// Luster Dragon, Luster Dragon #2 and Sky Dragon (WIND Dragons of Levels 4 to 6) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (88819587, MZONE),
    (88819587, MZONE),
    (88819587, GRAVE),
    (88819587, GRAVE),
    (11091375, DECK),
    (11091375, DECK),
    (17658803, DECK),
    (95288024, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_BOTH_SIDE);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_EVENT_PLAYER);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_DECK>);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

// The Lua tests a global `chkc` here (nil), so a targeted check is never refused.
fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(|c, duel| plfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_target(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e))
        && duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, original_race: u64, original_attr: u32, original_lv: u32) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.is_original_race(duel, original_race)
        && c.is_original_attribute(duel, original_attr)
        && c.is_level_between(duel, original_lv + 1, original_lv + 3)
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster_card(duel)
        && c.is_faceup(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| {
                spfilter(
                    sc,
                    duel,
                    e,
                    tp,
                    c.get_original_race(duel),
                    c.get_original_attribute(duel),
                    c.get_original_level(duel),
                )
            },
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spcostfilter(c, duel, e, tp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    if let Some(sc) = duel
        .select_matching_card(tp, |c, duel| spcostfilter(c, duel, e, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?
        .get_first()
    {
        duel.send_to_grave(sc, REASON_COST)?;
        e.set_chain_data(duel, ORIGINAL_RACE, sc.get_original_race(duel) as i64);
        e.set_chain_data(duel, ORIGINAL_ATTRIBUTE, sc.get_original_attribute(duel).into());
        e.set_chain_data(duel, ORIGINAL_LEVEL, sc.get_original_level(duel).into());
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let race = e.get_chain_data(duel, ORIGINAL_RACE).unwrap_or(0) as u64;
    let attr = e.get_chain_data(duel, ORIGINAL_ATTRIBUTE).unwrap_or(0) as u32;
    let level = e.get_chain_data(duel, ORIGINAL_LEVEL).unwrap_or(0) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, race, attr, level),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
