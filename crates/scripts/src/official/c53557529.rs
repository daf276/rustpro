// Whirlwind Fur Hire
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_MOVE;

const ID: u32 = 53557529;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
// In the hand with Fur Hire monsters in the Deck and Sangan on the other side; a copy in the Graveyard replaces Rafale's
// discard (Rafale on the field, the opponent's Sangan to negate).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (93850652, DECK),
    (28004531, DECK),
    (47647354, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (38916526, MZONE),
];
// The cost banish carries no REASON_REPLACE, so no check singles it out.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_MOVE)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_COST_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target_range(duel, 1, 0);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_able_to_remove_as_cost(duel));
    e2.set_value(duel, Value::CostReplace(repval));
    e2.set_cost_replace_operation(duel, |duel, base, _| {
        duel.remove(base.get_handler(duel), POS_FACEUP, REASON_COST)?;
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn lv4spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && spfilter(c, duel, e, tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| lv4spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g1 = duel.select_matching_card(tp, |c, duel| lv4spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g1.is_empty()
            && duel.special_summon(g1, 0, tp, tp, false, false, POS_FACEUP)? > 0
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.get_field_group_count(tp, 0, LOCATION_MZONE) >= 2
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g2 = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g2.is_empty() {
                duel.break_effect()?;
                duel.special_summon(g2, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_FUR_HIRE]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn repval(duel: &Duel, _: EffectId, call: CostReplaceCall) -> bool {
    let c = call.args.e.get_handler(duel);
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_faceup(duel) && c.is_controler(duel, call.args.tp) && c.is_location(duel, LOCATION_MZONE)
}
