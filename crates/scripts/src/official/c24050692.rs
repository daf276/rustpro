// Ogdoadic Water Lily
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 24050692;

// Four different Reptiles in the GY and Ogdoabyss in the Deck to send as the fifth; only this card summons Nauya.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (36010310, GRAVE),
    (62405028, GRAVE),
    (62405028, GRAVE),
    (35998832, GRAVE),
    (34172284, GRAVE),
    (97565997, DECK),
    (97565997, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(62405028)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_race(duel, RACE_REPTILE) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_race(duel, RACE_REPTILE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty()
        || duel.send_to_grave(g, REASON_EFFECT)? == 0
        || !duel
            .get_operated_group()
            .get_first()
            .is_some_and(|c| c.is_location(duel, LOCATION_GRAVE))
    {
        return Ok(());
    }
    let rg = duel.get_matching_group(|c, duel| c.is_race(duel, RACE_REPTILE), tp, LOCATION_GRAVE, 0, None);
    let ct = rg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if ct >= 5
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
