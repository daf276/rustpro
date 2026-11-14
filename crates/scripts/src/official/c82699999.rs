// Live☆Twin Lil-la Sweet
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 82699999;

pub const LISTED_SERIES: &[u16] = &[SET_LIVE_TWIN, SET_KI_SIKIL];

// Live☆Twin Ki-sikil on the field and in the hand, Sunny's Snitch to activate, and the opponent's Mystical Space Typhoons to chain to them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (36326160, MZONE),
    (36326160, HAND),
    (37582948, HAND),
    (37582948, HAND),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, distg);
    e1.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_KI_SIKIL])),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ch = i32::from(duel.get_current_chain_full(true)) - 1;
    ch > 0
        && a.ep == 1 - a.tp
        && duel.is_chain_disablable(a.ev as u8)
        && chain::is_triggering_player(duel, ch as u8, a.tp)
        && chain::is_triggering_setcode(duel, ch as u8, &[SET_LIVE_TWIN])
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(
            duel,
            EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT,
        );
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_absolute_range(duel, tp, 1, 0);
        e1.set_target_card(duel, |duel, _, c| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_FIEND)
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}
