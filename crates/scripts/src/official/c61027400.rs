// Gunkan Suship Ikura
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 61027400;
pub const LISTED_NAMES: &[u32] = &[CARD_SUSHIP_SHARI];

// In the hand with Gunkan Suship Shari on the field to Special Summon from, and more in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_SUSHIP_SHARI, MZONE),
    (CARD_SUSHIP_SHARI, DECK),
    (CARD_SUSHIP_SHARI, DECK),
    (CARD_SUSHIP_SHARI, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, exctg);
    e2.set_operation(duel, excop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_SUSHIP_SHARI)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn exctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) > 2);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn excop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) < 3 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 3)?;
    let excfilter = |c: CardId, duel: &Duel| {
        c.is_code(duel, CARD_SUSHIP_SHARI) && (c.is_able_to_hand(duel) || c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
    };
    let g = duel.get_decktop_group(tp, 3).filter(duel, excfilter, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |_, duel| tc.is_can_be_special_summoned(duel, e, 0, tp, false, false) && duel.get_location_count(tp, LOCATION_MZONE) > 0,
            |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 3),
        )
    } else {
        duel.shuffle_deck(tp)
    }
}
