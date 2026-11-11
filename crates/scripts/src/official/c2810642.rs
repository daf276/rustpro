// PSY-Framegear Beta
use super::c1697104::summon_with_driver;
use super::c74203495::spfilter;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 2810642;

pub const LISTED_NAMES: &[u32] = &[CARD_PSYFRAME_DRIVER];

// In the hand with PSY-Frame Driver in the Deck, against attackers on the opponent's field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_PSYFRAME_DRIVER, DECK),
    (CARD_PSYFRAME_DRIVER, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
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
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_attacker().is_some_and(|ac| ac.get_controler(duel) != tp)
        && (duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 || duel.is_player_affected_by_effect(tp, CARD_PSYFRAME_LAMBDA).is_some())
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let attacker = duel.get_attacker();
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && attacker.is_some_and(|ac| ac.is_relate_to_battle(duel))
            && e.get_handler(duel).is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_target_card(attacker.map(CardSet::from).unwrap_or_default());
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        2,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    duel.set_operation_info(0, CATEGORY_DESTROY, attacker.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if summon_with_driver::<ID>(duel, a, spfilter)?
        && let Some(dc) = duel.get_first_target()
        && dc.is_relate_to_effect(duel, a.e)
        && duel.destroy(dc, REASON_EFFECT)? != 0
    {
        duel.break_effect()?;
        duel.skip_phase(1 - a.tp, PHASE_BATTLE, RESET_PHASE | PHASE_BATTLE_STEP, 1);
    }
    Ok(())
}
