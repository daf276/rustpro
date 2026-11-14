// Gladiator Beast Dragases
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 62000467;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_set_card(duel, &[SET_GLADIATOR])
    }
}

// On the field after battle against 0 ATK monsters of the opponent's (Thousand-Eyes Idol), with few monsters of its own for two
// free zones, and Gladiator Beast Andal and Hoplomus (different names) in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (90582719, DECK),
    (90582719, DECK),
    (90582719, DECK),
    (4253484, DECK),
    (4253484, DECK),
    (4253484, DECK),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| indtg(c, duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_value(duel, 1);
    e2.set_condition(duel, actcon);
    c.register_effect(duel, e2);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_count_limit(duel, 1, ID);
    e6.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e6.set_cost(duel, spcost);
    e6.set_target(duel, sptg);
    e6.set_operation(duel, spop);
    c.register_effect(duel, e6);
}

fn indtg(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && duel.get_attacker() == Some(c)
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_attacker()
        .is_some_and(|tc| tc.is_controler(duel, tp) && tc.is_set_card(duel, &[SET_GLADIATOR]))
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_extra_as_cost(duel));
    }
    duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKTOP, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 125, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
        if e.get_handler(duel).get_sequence(duel) < 5 {
            ft += 1;
        }
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(ft > 1
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 2);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let mut g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if g.len() >= 2 && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let Some(tc1) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        let code = tc1.get_code(duel);
        g.remove_matching(duel, |c, duel| c.is_code(duel, code), None);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let Some(tc2) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        duel.special_summon_step(tc1, 125, tp, tp, false, false, POS_FACEUP)?;
        duel.special_summon_step(tc2, 125, tp, tp, false, false, POS_FACEUP)?;
        for tc in [tc1, tc2] {
            let code = tc.get_original_code(duel);
            tc.register_flag_effect(duel, code, RESET_EVENT + RESETS_STANDARD_DISABLE, 0, 0);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
