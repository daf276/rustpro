// Destiny HERO - Denier
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 16605586;
const MALICIOUS: u32 = 9411399;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and in the GY, with Destiny HERO - Malicious on the field, in the Deck, in the GY and banished.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (MALICIOUS, MZONE),
    (MALICIOUS, DECK),
    (MALICIOUS, GRAVE),
    (MALICIOUS, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit_full(duel, 1, (ID, 1), EFFECT_COUNT_CODE_DUEL);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO])
        && c.is_monster(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE | LOCATION_DECK))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let locations = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, 0, locations as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(filter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if tc.is_location(duel, LOCATION_DECK) {
        duel.shuffle_deck(tp)?;
        duel.move_to_deck_top(tc.into())?;
    } else {
        duel.hint_selection(g);
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    }
    if !tc.is_location(duel, LOCATION_EXTRA) {
        duel.confirm_decktop(tp, 1)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel) && c.is_set_card(duel, &[SET_DESTINY_HERO]) && !c.is_code(duel, ID)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spfilter, a.tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
