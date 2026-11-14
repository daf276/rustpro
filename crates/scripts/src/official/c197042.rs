// Exosister Returnia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 197042;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];

// Set and in the hand, with Exosister Elis and Stella on the field to Xyz Summon Exosister Mikailis.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (16474916, MZONE),
    (43863925, MZONE),
    (42741437, EXTRA),
    (42741437, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, rmcon);
    e1.set_cost(duel, Cost::pay_lp::<800>);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(|duel, c| !c.is_xyz_summoned(duel)));
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    !g.is_empty() && g.filter_count(duel, aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_EXOSISTER])), None) == g.len()
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE)
            && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_xyz_summonable(duel, None, None)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) || duel.remove(tc, POS_FACEUP, REASON_EFFECT)? < 1 {
        return Ok(());
    }
    let xg = duel.get_matching_group(spfilter, tp, LOCATION_EXTRA, 0, None);
    let rg = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    let b1 = !xg.is_empty();
    let b2 = !rg.is_empty() && duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) > 0;
    if !(b1 || b2) || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])? {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = xg.select(duel, tp, 1, 1, None)?;
            if let Some(sc) = g.get_first() {
                duel.xyz_summon(tp, sc, None, None)?;
            }
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let g = rg.select(duel, tp, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection_full(g, true);
                duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
