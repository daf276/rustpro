// Dragonmaid Hospitality
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 78231355;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGONMAID];
// In the hand with Dragonmaids (Chamber and Parlor) to Special Summon and of another Level and the same Attribute in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (32600024, HAND), (88453933, DECK), (32600024, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGONMAID]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn gyfilter(c: CardId, duel: &Duel, tc: CardId) -> bool {
    c.is_set_card(duel, &[SET_DRAGONMAID])
        && c.is_monster(duel)
        && c.is_able_to_grave(duel)
        && c.is_attribute(duel, tc.get_attribute(duel))
        && !c.is_level(duel, &[tc.get_level(duel)])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_HAND) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| filter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0
    {
        let g = duel.get_matching_group(|c, duel| gyfilter(c, duel, tc), tp, LOCATION_DECK, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
