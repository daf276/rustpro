// Elegant Egotist
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 90219263;

pub const LISTED_NAMES: &[u32] = &[CARD_HARPIE_LADY, CARD_HARPIE_LADY_SISTERS];

// In the hand with Harpie Lady (a Normal Monster) on the field, and Harpie Lady Sisters in the hand and the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_HARPIE_LADY, MZONE),
    (CARD_HARPIE_LADY, MZONE),
    (CARD_HARPIE_LADY_SISTERS, HAND),
    (CARD_HARPIE_LADY_SISTERS, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(CARD_HARPIE_LADY_SISTERS)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_HARPIE_LADY)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_HARPIE_LADY) || c.is_code(duel, CARD_HARPIE_LADY_SISTERS))
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, true, false, POS_FACEUP)? != 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}
