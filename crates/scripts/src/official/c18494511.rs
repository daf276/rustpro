// Icejade Ran Aegirine
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 18494511;
const ICEJADE_CENOTE_ENION_CRADLE: u32 = 7142724;

pub const LISTED_NAMES: &[u32] = &[ICEJADE_CENOTE_ENION_CRADLE, ID + 1];

// In the hand, with copies to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_CODE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, ICEJADE_CENOTE_ENION_CRADLE as i32);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_count(duel) > 0);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_discardable(duel) && (c.is_set_card(duel, &[SET_ICEJADE]) || (c.is_monster(duel) && c.is_attribute(duel, ATTRIBUTE_WATER)))
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand_full(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD, Some(c.into()))?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let stats = MonsterStats::new(SET_ICEJADE, TYPES_TOKEN, 0, 0, 3, RACE_AQUA, ATTRIBUTE_WATER);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, ID + 1, stats)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        let Some(token) = duel.create_token(tp, ID + 1) else {
            return Ok(());
        };
        duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_absolute_range(duel, tp, 1, 0);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_attribute(duel, ATTRIBUTE_WATER)
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e1, true);
        let e2 = aux::create_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| !c.is_attribute(duel, ATTRIBUTE_WATER));
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e2, true);
        duel.special_summon_complete()?;
    }
    Ok(())
}
