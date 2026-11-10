// Redox, Dragon Ruler of Boulders
use super::c26400609::{add_banished_search_effect, add_summon_and_return_effects};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 90411554;

// Copies in the hand, the Graveyard and the Deck (to be banished as a cost, then searched), with an EARTH Normal Monster and
// Red-Eyes Black Dragon (Dragon) as the cost cards and the Graveyard monster to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, DECK),
    (ID, DECK),
    (74677422, HAND),
    (549481, HAND),
    (74677422, DECK),
    (74677422, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_and_return_effects::<ID, ATTRIBUTE_EARTH>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_EARTH>);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    add_banished_search_effect::<ID, ATTRIBUTE_EARTH>(duel, c);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp)
            && chkc.is_location(duel, LOCATION_GRAVE)
            && chkc.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let filter = |c: CardId, duel: &Duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
