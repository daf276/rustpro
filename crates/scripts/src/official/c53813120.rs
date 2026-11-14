// Radiant Typhoon Mandate
use super::c20508881::add_set_effect;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53813120;

pub const LISTED_SERIES: &[u16] = &[SET_RADIANT_TYPHOON];
pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON];
// Quick-Play Spells (Mystical Space Typhoon twice, Radiant Typhoon Chant) in the GY to shuffle, a Mystical Space Typhoon in
// the hand to activate and one set to destroy this card with, and an Effect Monster of the opponent's to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_MYSTICAL_SPACE_TYPHOON, GRAVE),
    (CARD_MYSTICAL_SPACE_TYPHOON, GRAVE),
    (67115133, GRAVE),
    (CARD_MYSTICAL_SPACE_TYPHOON, HAND),
    (CARD_MYSTICAL_SPACE_TYPHOON, SZONE),
    (22912101, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_handler(duel).is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON))
    });
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    add_set_effect(duel, c, aux::stringid(ID, 2));
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_quick_play_spell(duel) && c.is_able_to_deck(duel) && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_RADIANT_TYPHOON]), 1, None),
        false,
    )
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tdfilter(c, duel, e), tp, LOCATION_GRAVE, 0, None);
    let radiant = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_RADIANT_TYPHOON]);
    if chk == 0 {
        return Ok(g.len() >= 3 && g.is_exists(duel, radiant, 1, None));
    }
    let tg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_TODECK)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), tg.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, None, 0, tp, 300);
    duel.set_possible_operation_info(0, CATEGORY_DEFCHANGE, None, 0, tp, 300);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut break_chk = false;
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty() && duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 {
        break_chk = true;
        duel.shuffle_deck(tp)?;
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    if break_chk {
        duel.break_effect()?;
    }
    let c = e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WIND));
    e1.set_value(duel, 300);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    duel.register_effect(e2, tp);
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        tc.negate_effects(duel, a.e.get_handler(duel), 0, true);
    }
    Ok(())
}
