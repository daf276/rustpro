// Red Nova
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 21142671;

// In the hand and on the field, with Red Dragon Archfiend and a Level 3 Tuner (Stinger) on the field and Red Nova Dragon
// in the Extra Deck for Synchro Summons with two Tuners, and Dokuroyaiba (a FIRE Fiend Normal Monster) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (57649113, MZONE),
    (57649113, MZONE),
    (97489701, EXTRA),
    (97489701, EXTRA),
    (30325729, DECK),
    (30325729, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, spdcon);
    e2.set_target(duel, spdtg);
    e2.set_operation(duel, spdop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO) && c.is_level_above(duel, 8) && c.is_race(duel, RACE_DRAGON)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn spdcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let reason_card = c.get_reason_card(duel);
    // Lua errors on a nil reason card
    unported!(
        duel,
        c.is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO && reason_card.is_none()
    );
    c.is_location(duel, LOCATION_GRAVE)
        && a.r == REASON_SYNCHRO
        && reason_card.is_some_and(|rc| rc.is_has_effect(duel, EFFECT_MULTIPLE_TUNERS).is_some())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_race(duel, RACE_FIEND)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
