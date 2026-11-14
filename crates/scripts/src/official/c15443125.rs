// Spright Starter
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 15443125;

pub const LISTED_SERIES: &[u16] = &[SET_SPRIGHT];

// Spright Blue and Jet in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (76145933, DECK), (76145933, DECK), (13533678, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(e: EffectId, tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| c.is_set_card(duel, &[SET_SPRIGHT]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, filter(a.e, a.tp), LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(tc) = duel
            .select_matching_card(tp, filter(e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
            .get_first()
            && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        {
            let atk = tc.get_text_attack(duel);
            if atk > 0 {
                let lp = duel.get_lp(tp);
                duel.set_lp(tp, lp - atk);
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !(c.is_level(duel, &[2]) || c.is_rank(duel, &[2]) || c.is_link(duel, &[2]))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
