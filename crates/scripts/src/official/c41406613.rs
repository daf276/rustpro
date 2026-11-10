// Reign-Beaux, Overking of Dark World
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 41406613;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Graveyard to return Renge from the field, in the hand to be discarded by Dark World Dealings from either side,
// with Grapha in the Deck to search and Renge and Zure to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (60606759, MZONE),
    (74117290, HAND),
    (74117290, HAND | OPPONENT),
    (34230233, DECK),
    (60606759, DECK),
    (7459013, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, super::c34230233::spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition_mut(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn spcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_WORLD]) && c.is_level_below(duel, 7) && c.is_able_to_hand_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    super::c34230233::spcon_with(duel, a, spcfilter)
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    super::c34230233::sptg_with(duel, a, spcfilter)
}

fn condition(duel: &mut Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let opponent_discard = a.rp == 1 - tp && c.is_previous_controler(duel, tp);
    a.e.set_label(duel, i64::from(opponent_discard));
    c.is_previous_location(duel, LOCATION_HAND) && a.r & (REASON_EFFECT + REASON_DISCARD) == REASON_EFFECT + REASON_DISCARD
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_WORLD]) && !c.is_code(duel, ID) && c.is_level_above(duel, 5) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    if e.get_label(duel) == 1 {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    } else {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARK_WORLD])
        && c.is_level_below(duel, 4)
        && ((duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
            || (duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
                && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    if e.get_label(duel) == 0 {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let sg = duel.get_matching_group(filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if sg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let b2 = duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
        && sc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    let target_player = if op == Some(1) { tp } else { 1 - tp };
    duel.break_effect()?;
    duel.special_summon(sc, 0, tp, target_player, false, false, POS_FACEUP)?;
    Ok(())
}
