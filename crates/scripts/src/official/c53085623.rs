// Mermail Shadow Squad
use super::c12975671::limit_extra_deck_to_water;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 53085623;

pub const LISTED_NAMES: &[u32] = &[ID];
// On the field and in the hand, with Mermail Abyssteus and Mermail Abysspike in the hand to discard it as a cost, and
// Mermail Abysspike in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (22446869, HAND),
    (58471134, HAND),
    (58471134, DECK),
    (58471134, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, lvcost);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable(duel) && c.is_able_to_grave_as_cost(duel)
}

fn lvcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel) && !c.is_level(duel, &[7]) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_faceup(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(lvfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(g), g.len() as u8, tp, 7);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(lvfilter, a.tp, LOCATION_MZONE, 0, None);
    let c = a.e.get_handler(duel);
    for tc in g.iter() {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 7);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let attribute: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_ATTRIBUTE);
    a.e.get_handler(duel).is_reason(duel, REASON_COST)
        && a.re.is_some_and(|re| re.is_activated(duel) && re.is_monster_effect(duel))
        && attribute & ATTRIBUTE_WATER > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_set_card(duel, &[SET_ATLANTEAN, SET_MERMAIL])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    limit_extra_deck_to_water(duel, e.get_handler(duel), tp, aux::stringid(ID, 2));
    Ok(())
}
