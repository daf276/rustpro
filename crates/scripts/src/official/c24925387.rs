// Adamancipator Conductor
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 24925387;

pub const LISTED_SERIES: &[u16] = &[SET_ADAMANCIPATOR];
// In the hand and on the field, with Adamancipator Researcher in the hand to place and in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (85914562, HAND),
    (85914562, DECK),
    (85914562, DECK),
    (85914562, DECK),
    (48519867, DECK),
    (48519867, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, excavtg);
    e2.set_operation(duel, excavop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ADAMANCIPATOR]) && c.is_able_to_deck_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, spcostfilter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    duel.confirm_cards(1 - tp, g)?;
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_COST)?;
    duel.confirm_decktop(tp, 1)?;
    Ok(false)
}

fn excavtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) >= 5);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn excavspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_ROCK) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn excavop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) == 0 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 5)?;
    let excavg = duel.get_decktop_group(tp, 5);
    let mut remaining_count = excavg.len();
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        let g = excavg.filter(duel, |c, duel| excavspfilter(c, duel, e, tp), None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            if !sg.is_empty() {
                duel.disable_shuffle_check(true);
                remaining_count -= 1;
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    if remaining_count > 0 {
        if duel.get_field_group_count(tp, LOCATION_DECK, 0) > remaining_count {
            duel.move_to_deck_bottom_count(remaining_count, tp)?;
        }
        duel.sort_deckbottom(tp, tp, remaining_count as u16)?;
    }
    Ok(())
}
