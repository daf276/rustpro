// Number C102: Archfiend Seraph
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 67173574;
pub const XYZ_NUMBER: u32 = 102;
const STAR_ARK_105: u32 = 49678559;

pub const LISTED_NAMES: &[u32] = &[STAR_ARK_105];

// In the Extra Deck, Xyz Summoned with four Level 5 LIGHT Normal Monsters, and the opponent's Smashing Ground and Fissure to destroy it.
// Number 102: Star Ark 105 is not ported, so the disable effect has no scenario.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (35565537, MZONE),
    (86088138, MZONE),
    (90876561, MZONE),
    (2971090, MZONE),
    (97169186, HAND | OPPONENT),
    (66788016, HAND | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Message(MSG_SELECT_EFFECTYN),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 4;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    duel.enable_global_flag(GLOBALFLAG_DETACH_EVENT);
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_code(duel, EVENT_DETACH_MATERIAL);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_code(duel, STAR_ARK_105), 1, None)
    });
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 2, REASON_EFFECT));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    c.remove_overlay_card(duel, tp, 2, 2, REASON_EFFECT)?;
    Ok(true)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel));
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1500);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.get_attack(duel) > 0
        && tc.is_controler(duel, 1 - tp)
    {
        for (code, value) in [
            (EFFECT_SET_ATTACK_FINAL, Some(0)),
            (EFFECT_DISABLE, None),
            (EFFECT_DISABLE_EFFECT, None),
        ] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            if let Some(value) = value {
                e1.set_value(duel, value);
            }
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
