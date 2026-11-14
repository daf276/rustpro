// Mathmech Addition
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 80965043;

// Copies in the hand, with Dark Magicians on the field to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (46986414, MZONE), (46986414, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_effect::<ID, false>(duel, c);
}

// The effect of Mathmech Addition and Subtraction, which only differ in the ATK change and Subtraction's BreakEffect.
pub fn add_summon_effect<const ID: u32, const SUBTRACT: bool>(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop::<ID, SUBTRACT>);
    c.register_effect(duel, e1);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop<const ID: u32, const SUBTRACT: bool>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e0.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e0.set_reset(duel, RESET_PHASE | PHASE_END);
    e0.set_description(duel, aux::stringid(ID, 1));
    e0.set_target_range(duel, 1, 0);
    e0.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_CYBERSE)
    });
    duel.register_effect(e0, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_CYBERSE));
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel))
    {
        if SUBTRACT {
            duel.break_effect()?;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, if SUBTRACT { -1000 } else { 1000 });
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 3206);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_CANNOT_ATTACK);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}
