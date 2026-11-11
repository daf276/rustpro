// Psi-Reflector
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 74644400;
const CRIMSON_BLADER_ASSAULT_MODE: u32 = 50604072;
const ASSAULT_SONIC_WARRIOR: u32 = 18711696;

pub const LISTED_NAMES: &[u32] = &[CARD_ASSAULT_MODE];

// A copy in the hand to Normal Summon and one on the field, with Assault Mode Activate in hand and Deck, and a card that
// lists it (Crimson Blader/Assault Mode) in the Deck and Assault Sonic Warrior in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_ASSAULT_MODE, HAND),
    (CARD_ASSAULT_MODE, HAND),
    (CARD_ASSAULT_MODE, DECK),
    (CRIMSON_BLADER_ASSAULT_MODE, DECK),
    (ASSAULT_SONIC_WARRIOR, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(ASSAULT_SONIC_WARRIOR)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_ASSAULT_MODE) || c.lists_code(duel, CARD_ASSAULT_MODE)) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_ASSAULT_MODE) && !c.is_public(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.lists_code(duel, CARD_ASSAULT_MODE)
        && !c.is_code(duel, ID)
        && c.is_level_above(duel, 1)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        let lv = duel.select_option(
            tp,
            &[
                aux::stringid(ID, 2),
                aux::stringid(ID, 3),
                aux::stringid(ID, 4),
                aux::stringid(ID, 5),
            ],
        )? + 1;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, lv);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
