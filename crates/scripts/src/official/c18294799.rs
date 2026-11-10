// Schwarzschild Infinity Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 18294799;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with the opponent's Dark Magician (2500 ATK) and Blue-Eyes White Dragon (Level 8 LIGHT Dragon) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (46986414, MZONE | OPPONENT),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(CARD_BLUEEYES_W_DRAGON)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| {
            !c.is_summon_location(duel, LOCATION_EXTRA) || (c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_XYZ))
        }),
    );
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        || duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_attack_above(duel, 2000)),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_XYZ))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn dspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[8])
        && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
        && c.is_race(duel, RACE_DRAGON)
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(|c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        Some(c.into()),
        2,
        tp,
        (LOCATION_HAND | LOCATION_DECK) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    if !c.is_relate_to_effect(duel, e) || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    if duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
        {
            sc.negate_effects(duel, c, 0, false);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
