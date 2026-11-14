// Summoner of the Sacred Beasts
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 22734799;

pub const LISTED_SERIES: &[u16] = &[SET_SACRED_BEAST];

const HAND_OR_GRAVE: u32 = LOCATION_HAND | LOCATION_GRAVE;

// Copies in the hand, on the field and in the Graveyard for the three effects, and Sacred Beast monsters other than
// Level 8 in the hand and Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (23856331, HAND),
    (59138498, HAND),
    (50251045, GRAVE),
    (96345184, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, sptg::<LOCATION_HAND>);
    e1.set_operation(duel, spop::<LOCATION_HAND>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, discardcost);
    e2.set_target(duel, sptg::<HAND_OR_GRAVE>);
    e2.set_operation(duel, spop::<HAND_OR_GRAVE>);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, sptg::<LOCATION_GRAVE>);
    e3.set_operation(duel, spop::<LOCATION_GRAVE>);
    c.register_effect(duel, e3);
}

fn discardcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Cost::discard(duel, a, 0);
    }
    Cost::discard(duel, a, 1)?;
    let oc = duel.get_operated_group();
    if let Some(oc) = oc.get_first() {
        oc.create_effect_relation(duel, a.e);
    }
    a.e.set_label_object_group(duel, Some(oc));
    Ok(false)
}

fn discardfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_HAND, 0, 1, Some(c.into()))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST])
        && !c.is_level(duel, &[8])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, true, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg<const LOCATION: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(false);
        }
        return Ok(if LOCATION == LOCATION_HAND {
            duel.is_existing_matching_card(|c, duel| discardfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
        } else {
            duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION,
                0,
                1,
                Some(e.get_handler(duel).into()),
            )
        });
    }
    if LOCATION == LOCATION_HAND {
        duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION as i32);
    Ok(false)
}

fn spop<const LOCATION: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if LOCATION == LOCATION_HAND
        && duel.discard_hand(tp, |c, duel| discardfilter(c, duel, e, tp), 1, 1, REASON_EFFECT | REASON_DISCARD)? < 0
    {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let mut exc = None;
    if LOCATION == HAND_OR_GRAVE {
        let cost_card = e.get_label_object_group(duel).and_then(|g| g.get_first());
        exc = cost_card
            .filter(|cost_card| cost_card.is_relate_to_effect(duel, e))
            .map(CardSet::from);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION, 0, 1, 1, exc)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, false, true, POS_FACEUP_DEFENSE)? > 0
    {
        sc.complete_procedure(duel);
    }
    Ok(())
}
