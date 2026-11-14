// Malefic Paradox Gear
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 48829461;

pub const LISTED_SERIES: &[u16] = &[SET_MALEFIC];

const PARALLEL_GEAR: u32 = 74509280;

pub const LISTED_NAMES: &[u32] = &[PARALLEL_GEAR];

// On the field with Umi in the hand as the Field Spell, and Malefic Parallel Gear in the Deck to Special Summon and add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (22702055, HAND),
    (PARALLEL_GEAR, DECK),
    (PARALLEL_GEAR, DECK),
    (PARALLEL_GEAR, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_GRAVE | LOCATION_MZONE);
    e2.set_code(duel, ID);
    e2.set_count_limit(duel, 1, (ID, 1));
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, _: &Args) -> bool {
    duel.is_existing_matching_card(CardId::is_faceup, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALEFIC]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, chk: bool) -> bool {
    c.is_code(duel, PARALLEL_GEAR)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (!chk || duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, Some(c.into())))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > -1
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, true), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, false), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let ag = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !ag.is_empty() {
                duel.break_effect()?;
                duel.send_to_hand(ag, tp, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, ag)?;
            }
        }
    }
    Ok(())
}
