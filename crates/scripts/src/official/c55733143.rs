// Mimighoul Charm
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 55733143;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// In the hand to activate, with Mimighoul Fairy face-down on the opponent's field to Flip and in the hand to give them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (43066927, MZONE | OPPONENT),
    (43066927, MZONE | OPPONENT),
    (43066927, MZONE | OPPONENT),
    (43066927, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_FLIP);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 2, ID);
    e1.set_condition(duel, |duel, a| {
        let tp = a.tp;
        duel.is_main_phase()
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |c, duel| spconfilter(c, duel, tp), 1, None)
    });
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && c.is_set_card(duel, &[SET_MIMIGHOUL])
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID));
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok((duel.is_player_can_special_summon_any(tp) || duel.is_player_can_remove(tp, None))
            && duel.is_existing_matching_card(CardId::is_facedown, tp, 0, LOCATION_EXTRA, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_facedown, tp, 0, LOCATION_EXTRA, None);
    if g.is_empty() {
        return Ok(());
    }
    let Some(tc) = g.random_select(duel, tp, 1).get_first() else {
        return Ok(());
    };
    duel.confirm_cards(tp, tc)?;
    if duel.get_location_count_from_ex(tp, tp, None, tc) > 0
        && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3302);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        duel.special_summon_complete()?;
    } else {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
