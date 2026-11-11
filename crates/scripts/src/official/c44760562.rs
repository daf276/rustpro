// Mannadium Meek
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 44760562;

pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST, ID];

// In the hand and the Deck, with Visas Starfrost and Tearlaments Reinoheart (1500 ATK, 2100 DEF) on the field to Special Summon
// from the hand, and Dark Hole in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (CARD_VISAS_STARFROST, MZONE),
    (73956664, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spproccon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_BATTLE | REASON_EFFECT));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && (c.is_code(duel, CARD_VISAS_STARFROST) || (c.is_monster(duel) && c.is_attack(duel, &[1500]) && c.is_defense(duel, &[2100])))
}

fn spproccon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(cfilter, tp, LOCATION_ONFIELD, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = duel
        .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, 2);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
