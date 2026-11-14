// Fiendsmith in Paradise
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 99989863;

pub const LISTED_SERIES: &[u16] = &[SET_FIENDSMITH];

// In hand with Fiendsmith's Desirae on the field to target, and in the GY with Fiendsmith Engraver and Fiendsmith's Desirae
// in the Deck and Extra Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (82135803, MZONE),
    (ID, GRAVE),
    (60764609, DECK),
    (60764609, DECK),
    (82135803, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn trgtfilter(c: CardId, duel: &Duel, rc: CardId) -> bool {
    c.is_level_above(duel, 7)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_FIEND)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            aux::TRUE,
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(CardSet::from_iter([c, rc])),
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && trgtfilter(chkc, duel, c));
    }
    let filter = |tc, duel: &Duel| trgtfilter(tc, duel, c);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel
        .select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .unwrap_or_default();
    let tgg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(g.union(c.into())));
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(tgg), tgg.len() as u8, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let exceptions = CardSet::from_iter([tc, e.get_handler(duel)]);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(exceptions));
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FIENDSMITH]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK | LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK | LOCATION_EXTRA)
}
