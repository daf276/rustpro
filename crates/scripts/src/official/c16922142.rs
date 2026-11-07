// Radiant Typhoon Krosea
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 16922142;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_MYSTICAL_SPACE_TYPHOON];
// A copy and Mystical Space Typhoon (a Quick-Play Spell) in the hand, and Mystical Space Typhoon and Radiant Typhoon Swen in
// the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_MYSTICAL_SPACE_TYPHOON, HAND),
    (CARD_MYSTICAL_SPACE_TYPHOON, DECK),
    (80538047, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(CardId::is_spell_trap, a.e.get_handler_player(duel), 0, LOCATION_ONFIELD, 1, None)
    });
    e1a.set_value(duel, 1);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, quickplaycon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 1));
    e3a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_count_limit(duel, 1, (ID, 1));
    e3a.set_target(duel, thtg);
    e3a.set_operation(duel, thop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
}

// The condition of the Radiant Typhoon monsters that react to a Quick-Play Spell being activated.
pub fn quickplaycon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_spell_effect(duel) && re.is_active_type(duel, TYPE_QUICKPLAY))
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_RADIANT_TYPHOON]) || c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON))
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_RADIANT_TYPHOON]), None) <= 1
            && sg.filter_count(duel, |c, duel| c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON), None) <= 1,
        false,
    )
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !g.is_empty() {
        let sg = aux::select_unselect_group(duel, g, 1, 2, &rescon, tp, HINTMSG_ATOHAND)?;
        if !sg.is_empty() {
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    register_wind_only(duel, a.e.get_handler(duel), tp, aux::stringid(ID, 2));
    Ok(())
}

// "You cannot Special Summon for the rest of this turn, except WIND monsters", registered by the card's effect.
pub fn register_wind_only(duel: &mut Duel, c: CardId, tp: u8, description: u64) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, description);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_attribute(duel, ATTRIBUTE_WIND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
}
