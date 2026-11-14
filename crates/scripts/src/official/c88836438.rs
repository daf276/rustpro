// Spright Smashers
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 88836438;

pub const LISTED_SERIES: &[u16] = &[SET_SPRINGANS, SET_THERION, SET_SPRIGHT];

// Spright Blue to banish, Springans Kitt in the Deck, Therion "King" Regulus in the GY, Psychic Kappa (Level 2) to banish with an opponent's card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (76145933, HAND),
    (76145933, GRAVE),
    (45484331, DECK),
    (45484331, DECK),
    (10604644, GRAVE),
    (7892180, MZONE),
    (7892180, MZONE),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(45484331),
    Check::SpecialSummonedCode(10604644),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, actcost);
    e1.set_operation(duel, actop);
    c.register_effect(duel, e1);
}

fn actcostfilter(c: CardId, duel: &Duel, tp: u8, dsp: bool, gg: CardSet, rg: Option<CardSet>) -> bool {
    if !(c.is_set_card(duel, &[SET_SPRINGANS]) || c.is_set_card(duel, &[SET_THERION]) || c.is_set_card(duel, &[SET_SPRIGHT]))
        || !c.is_able_to_remove_as_cost(duel)
        || !(c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
    {
        return false;
    }
    let others = |g: CardSet| !g.difference(c.into()).is_empty();
    (duel.get_mzone_count(tp, Some(c.into())) > 0 && (dsp || others(gg))) || rg.is_some_and(others)
}

fn spfilter(e: EffectId, tp: u8, set: u16) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| c.is_set_card(duel, &[set]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && c.is_faceup(duel) && (c.is_level(duel, &[2]) || c.is_rank(duel, &[2]) || c.is_link(duel, &[2]))
}

fn actcost(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let dsp = duel.is_existing_matching_card(spfilter(e, tp, SET_SPRINGANS), tp, LOCATION_DECK, 0, 1, None);
    let gg = duel.get_matching_group(spfilter(e, tp, SET_THERION), tp, LOCATION_GRAVE, 0, None);
    let rg = duel
        .is_existing_matching_card(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_ONFIELD, 1, None)
        .then(|| duel.get_matching_group(rmfilter, tp, LOCATION_MZONE, 0, None));
    let costfilter = |c, duel: &Duel| actcostfilter(c, duel, tp, dsp, gg, rg);
    let locations = LOCATION_HAND | LOCATION_GRAVE | LOCATION_MZONE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, locations, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let cg = duel.select_matching_card(tp, costfilter, tp, locations, 0, 1, 1, Some(c.into()))?;
    duel.remove(cg, POS_FACEUP, REASON_COST)?;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let op = duel.select_effect(
        tp,
        &[
            (ft > 0 && dsp, aux::stringid(ID, 1)),
            (ft > 0 && !gg.difference(cg).is_empty(), aux::stringid(ID, 2)),
            (rg.is_some_and(|rg| !rg.difference(cg).is_empty()), aux::stringid(ID, 3)),
        ],
    )?;
    let op = op.unwrap_or(0);
    e.set_label(duel, op as i64);
    match op {
        1 | 2 => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            let location = if op == 1 { LOCATION_DECK } else { LOCATION_GRAVE };
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
        }
        3 => {
            e.set_category(duel, CATEGORY_REMOVE);
            let targets = rg.map(|rg| rg.difference(cg));
            duel.set_operation_info(0, CATEGORY_REMOVE, targets, 2, PLAYER_ALL, LOCATION_ONFIELD as i32);
        }
        _ => e.set_category(duel, 0),
    }
    Ok(false)
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        op @ (1 | 2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = if op == 1 {
                duel.select_matching_card(tp, spfilter(e, tp, SET_SPRINGANS), tp, LOCATION_DECK, 0, 1, 1, None)?
            } else {
                let filter = aux::necro_valley_filter(spfilter(e, tp, SET_THERION));
                duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
            };
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        3 => {
            let rg1 = duel.get_matching_group(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_ONFIELD, None);
            if rg1.is_empty() {
                return Ok(());
            }
            let rg2 = duel.get_matching_group(rmfilter, tp, LOCATION_MZONE, 0, None);
            if rg2.is_empty() {
                return Ok(());
            }
            let rescon =
                |duel: &Duel, sg: CardSet, _, _| (sg.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) == 2, false);
            let g = aux::select_unselect_group(duel, rg1.union(rg2), 2, 2, &rescon, tp, HINTMSG_REMOVE)?;
            if !g.is_empty() {
                duel.hint_selection_full(g, true);
                duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
