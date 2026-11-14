// Gladiator Beast Equeste
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 57731460;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR_BEAST];
pub const LISTED_NAMES: &[u32] = &[ID];
// Summoned from the hand with Gladiator Beast Gistel or by a tag-out from the Deck, with Hoplomus to summon in turn and Andal in
// the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (26582143, HAND),
    (26582143, HAND),
    (4253484, MZONE),
    (4253484, DECK),
    (90582719, GRAVE),
    (90582719, GRAVE),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, aux::gbspcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_deck);
    e2.set_target(duel, |duel, a, chk, _| sptg(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp)));
    e2.set_operation(duel, |duel, a| spop(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), 103, true, 0));
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, count, tp, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 103, tp, false, false)
}

// The Gladiator Beasts' tag-out target: a free zone once this card left, and a monster to summon from the Deck.
pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0 && duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

// The tag-out operation; `checked` flags the summoned monster only when the summon succeeded, `count` is the flag's.
pub fn spop(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool, sumtype: u32, checked: bool, count: u16) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        let summoned = duel.special_summon(sc, sumtype, tp, tp, false, false, POS_FACEUP)? > 0;
        if summoned || !checked {
            let code = sc.get_original_code(duel);
            sc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, count);
        }
    }
    Ok(())
}
