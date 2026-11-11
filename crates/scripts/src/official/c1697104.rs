// PSY-Framegear Epsilon
use super::c38814750::{rmcon, rmop};
use super::c74203495::{spfilter, target};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, SZONE};

const ID: u32 = 1697104;

pub const LISTED_NAMES: &[u32] = &[CARD_PSYFRAME_DRIVER];

// In the hand with PSY-Frame Driver in the Deck, against Traps the opponent sets and activates freely.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_PSYFRAME_DRIVER, DECK),
    (CARD_PSYFRAME_DRIVER, DECK),
    (30459350, SZONE | OPPONENT),
    (30459350, SZONE | OPPONENT),
    (35316708, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, se, _, _, _, _| se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.ep != tp
        && a.re
            .is_some_and(|re| re.is_trap_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && (duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 || duel.is_player_affected_by_effect(tp, CARD_PSYFRAME_LAMBDA).is_some())
}

// The Lua of Alpha, Beta and Epsilon up to the end-phase banishing effect, which they share.
pub(super) fn summon_with_driver<const CODE: u32>(
    duel: &mut Duel,
    a: &Args,
    filter: fn(CardId, &Duel, EffectId, u8) -> bool,
) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(false);
    }
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || !c.is_relate_to_effect(duel, e) {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| filter(c, duel, e, tp));
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(false) };
    let fid = c.get_field_id(duel);
    duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)?;
    for sc in [tc, c] {
        sc.register_flag_effect_full(duel, CODE, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
    }
    duel.special_summon_complete()?;
    g.merge(c.into());
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, fid.into());
    e1.set_label_object_group(duel, Some(g));
    e1.set_condition_mut(duel, rmcon::<CODE>);
    e1.set_operation(duel, rmop::<CODE>);
    duel.register_effect(e1, tp);
    Ok(true)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if summon_with_driver::<ID>(duel, a, spfilter)?
        && duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
