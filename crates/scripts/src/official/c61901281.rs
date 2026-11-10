// Black Dragon Collapserpent
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 61901281;
const WHITE_DRAGON_WYVERBURSTER: u32 = 99234526;

pub const LISTED_NAMES: &[u32] = &[WHITE_DRAGON_WYVERBURSTER];

// In the hand with Mokey Mokey (LIGHT) in the Graveyard and on the field to banish and White Dragon Wyverburster in the Deck
// to search, and the opponent's Dark Hole to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (27288416, GRAVE),
    (27288416, GRAVE),
    (27288416, MZONE),
    (WHITE_DRAGON_WYVERBURSTER, DECK),
    (WHITE_DRAGON_WYVERBURSTER, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID, ATTRIBUTE_LIGHT, WHITE_DRAGON_WYVERBURSTER>(duel, c);
}

// Shared with White Dragon Wyverburster, which is the same card with the other attribute and name.
pub fn register<const CODE: u32, const ATTRIBUTE: u32, const SEARCHED: u32>(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit_full(duel, 1, CODE, EFFECT_COUNT_CODE_OATH);
    e2.set_condition(duel, spcon::<ATTRIBUTE>);
    e2.set_target(duel, sptg::<ATTRIBUTE>);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(CODE, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e3.set_target(duel, target::<SEARCHED>);
    e3.set_operation(duel, operation::<SEARCHED>);
    c.register_effect(duel, e3);
}

fn spfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && (duel.get_location_count(tp, LOCATION_MZONE) > 0 || (c.is_location(duel, LOCATION_MZONE) && c.get_sequence(duel) < 5))
}

fn spcon<const ATTRIBUTE: u32>(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = duel.get_matching_group(
        |c, duel| spfilter::<ATTRIBUTE>(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        None,
    );
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    ft > -1 && !rg.is_empty() && aux::select_unselect_group_check(duel, rg, 1, 1, &|_, _, _, _| (true, false))
}

fn sptg<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(
        |c, duel| spfilter::<ATTRIBUTE>(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        None,
    );
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn filter<const SEARCHED: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, SEARCHED) && c.is_able_to_hand(duel)
}

fn target<const SEARCHED: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter::<SEARCHED>, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation<const SEARCHED: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter::<SEARCHED>, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
