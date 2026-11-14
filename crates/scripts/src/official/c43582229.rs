// Freezing Chains of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 43582229;

pub const LISTED_SERIES: &[u16] = &[SET_ICE_BARRIER];

// In the hand with Ice Barrier monsters in the Graveyard to summon and on the field for the immunity.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (23950192, GRAVE),
    (23950192, GRAVE),
    (23950192, GRAVE),
    (9396662, MZONE),
    (70703416, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(23950192)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_condition(duel, |duel, a| {
        let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_ICE_BARRIER]);
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 3, None)
    });
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_ICE_BARRIER]));
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(duel, Value::Effect(unaval));
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && c.is_set_card(duel, &[SET_ICE_BARRIER]) && c.is_level_below(duel, 4)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 0))?
    {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    } else {
        e.set_category(duel, 0);
        e.set_property(duel, 0);
        e.clear_operation(duel);
    }
    Ok(false)
}

fn unaval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    let tc = te.get_owner(duel);
    tc.is_special_summoned(duel)
        && tc.is_summon_location(duel, LOCATION_EXTRA)
        && te.get_owner_player(duel) != e.get_handler_player(duel)
        && te.is_monster_effect(duel)
        && te.is_activated(duel)
        && te.get_activate_location(duel) == LOCATION_MZONE
}
