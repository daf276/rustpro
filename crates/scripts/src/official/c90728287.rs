// Ars Magna - "Philosophirum"
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 90728287;

// In hand to activate and in the Graveyard to return Purification Power Patron, with Ars Magna monsters on the field and in the
// Graveyard to banish and Sangan of the opponent to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (36270527, MZONE),
    (36270527, GRAVE),
    (62368221, GRAVE),
    (31822037, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, retsptg);
    e2.set_operation(duel, retspop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn banfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARS_MAGNA]) && c.is_monster(duel) && c.is_able_to_remove(duel) && c.is_faceup(duel)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && banfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(banfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, banfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_REMOVE, g, count, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, count, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty() && duel.remove(tg, POS_FACEUP, REASON_EFFECT)? > 0 {
        let max_negate_count = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), None);
        if max_negate_count == 0 {
            return Ok(());
        }
        let c = e.get_handler(duel);
        let exc = c.is_relate_to_effect(duel, e).then(|| CardSet::from(c));
        let g = duel.get_matching_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
            let dg = g.select(duel, tp, 1, max_negate_count as u8, None)?;
            if dg.is_empty() {
                return Ok(());
            }
            duel.hint_selection(dg);
            duel.break_effect()?;
            for dc in dg {
                dc.negate_effects(duel, c, RESET_PHASE | PHASE_END, true);
            }
        }
    }
    Ok(())
}

fn retfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel) && c.is_able_to_extra(duel)
}

fn retsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(retfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn retspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let Some(sc) = duel
        .select_matching_card(tp, retfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    if duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_EXTRA)
        && duel.get_location_count_from_ex(tp, tp, None, sc) > 0
        && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.break_effect()?;
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
