// Rookie Fur Hire
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 48214588;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
// In the hand with Beat (Level 3) on the field to Tribute for Rex, Freight (Level 2) in the Deck, and the reverse.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (93850652, MZONE),
    (28004531, MZONE),
    (28004531, DECK),
    (93850652, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_label(duel, 0);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_ATTACK,
        CounterFilter::Card(|duel, c| c.is_set_card(duel, &[SET_FUR_HIRE])),
    );
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.e.set_label(duel, 100);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_ATTACK) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_OATH);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_FUR_HIRE]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(false)
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.has_level(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, c.get_level(duel), e, tp),
            tp,
            LOCATION_DECK | LOCATION_HAND,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, lv: u32, e: EffectId, tp: u8) -> bool {
    let original = i64::from(c.get_original_level(duel));
    (original == i64::from(lv) + 1 || original == i64::from(lv) - 1)
        && c.is_set_card(duel, &[SET_FUR_HIRE])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| costfilter(c, duel, e, tp);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    let lv = g.get_first().map_or(0, |tc| tc.get_level(duel));
    e.set_label(duel, i64::from(lv));
    duel.release(g, REASON_COST)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_HAND) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = e.get_label(duel) as u32;
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, lv, e, tp),
        tp,
        LOCATION_DECK | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    e.set_label(duel, 0);
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
