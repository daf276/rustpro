// Live☆Twin Lil-la
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::MSG_PAY_LPCOST;

const ID: u32 = 73810864;

pub const LISTED_NAMES: &[u32] = &[ID];

// In hand with Live Twin Ki-sikil in the Deck to Special Summon, and an attacker to pay Life Points to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (36326160, DECK),
    (36326160, DECK),
    (36326160, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_PAY_LPCOST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_ATTACK_COST);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 0, 1);
    e3.set_cost_condition(duel, |duel, _, _, tp, _| {
        duel.check_lp_cost(tp, duel.get_player_effect(tp, ID).len() as u32 * 500)
    });
    e3.set_operation(duel, atop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, ID);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_target_range(duel, 0, 1);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KI_SIKIL]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_HAND, 0, 1, None)
            && !duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, 0, 1, Some(e.get_handler(duel).into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_HAND) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_HAND,
        POS_FACEUP,
    )
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.is_attack_cost_paid() != 2 {
        duel.pay_lp_cost(a.tp, 500)?;
        duel.attack_cost_paid();
    }
    Ok(())
}
