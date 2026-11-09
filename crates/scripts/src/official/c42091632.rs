// Dominus Spiral
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 42091632;

// In the hand, to be activated after Ash Blossom & Joyous Spring in the opponent's hand responds to a Raigeki from the
// opponent; Elfnote Lucina in the opponent's Graveyard is the monster to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (14558127, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e0.set_condition(duel, |duel, a| {
        duel.get_custom_activity_count(ID, 1 - a.e.get_handler_player(duel), ACTIVITY_CHAIN) > 0
    });
    c.register_effect(duel, e0);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn chainfilter(duel: &Duel, re: EffectId, _: u8, _: u8) -> bool {
    !(re.is_monster_effect(duel) && re.get_activate_location(duel) & (LOCATION_HAND | LOCATION_GRAVE) > 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_hand(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, 1, None));
    }
    let act_from_hand = e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_handler(duel).is_status(duel, STATUS_ACT_FROM_HAND);
    e.set_label(duel, i64::from(act_from_hand));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND | LOCATION_EXTRA)
        && !duel.is_existing_matching_card(CardId::is_trap, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.get_location_count_full(1 - tp, LOCATION_MZONE, 1 - tp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_existing_matching_card(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, 1 - tp)),
            tp,
            0,
            LOCATION_GRAVE,
            1,
            None,
        )
        && duel.select_yes_no(1 - tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(1 - tp, |c, duel| spfilter(c, duel, e, 1 - tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, 1 - tp, 1 - tp, false, false, POS_FACEUP)?;
        }
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_label(duel) == 1 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_target_range(duel, 1, 0);
        e1.set_value(
            duel,
            Value::EffectPlayer(|duel, _, re, _| {
                re.is_monster_effect(duel) && re.get_handler(duel).is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
            }),
        );
        duel.register_effect(e1, tp);
    }
    Ok(())
}
