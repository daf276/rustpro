// Angelechy Verdict
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 101402096;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];
// Set, with Angelechy Destrier on the field and Sangan of the opponent's to negate; in the Graveyard with Angelechy Problem in hand
// to send and Angelechy monsters in the Extra Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (55393975, MZONE),
    (26202165, MZONE | OPPONENT),
    (17782288, HAND),
    (17782288, HAND),
    (81797573, EXTRA),
    (28904860, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn countfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_monster_card(duel) && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let ct = duel.get_matching_group_count(countfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        if ct == 0 {
            return Ok(false);
        }
        if ct < 5 {
            return Ok(duel.is_existing_matching_card(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
        }
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(countfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if ct == 0 {
        return Ok(());
    }
    if ct < 5 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        let g = duel.select_matching_card(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            duel.hint_selection(tc);
            let c = a.e.get_handler(duel);
            tc.negate_effects(duel, c, 0, true);
        }
    } else {
        let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
        if !g.is_empty() {
            duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_able_to_grave(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_HAND | LOCATION_ONFIELD, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_grave(g, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_GRAVE) || super::c101402094::szone_count(duel, tp) == 0
    {
        return Ok(());
    }
    let codes = super::c101402094::stzone_codes(duel, tp);
    let plg = duel.get_matching_group(
        |c, duel| super::c101402094::spplfilter(c, duel, &codes),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if plg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let Some(plc) = plg.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    duel.break_effect()?;
    if duel.move_to_field(plc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
        super::c101402094::treat_as_continuous_spell(duel, e, plc);
    }
    Ok(())
}
