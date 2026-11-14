// Water Leviathan @Ignister
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 37061511;
const AI_RITUAL: u32 = 85327820;

pub const LISTED_NAMES: &[u32] = &[AI_RITUAL];

// Ritual Summoned by A.I.'s Ritual with Wizard @Ignister, with Link Spider in the Graveyard, and copies on the field the
// opponent's Elfnote Lucina attacks.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (AI_RITUAL, HAND),
    (AI_RITUAL, HAND),
    (3723262, MZONE),
    (3723262, MZONE),
    (98978921, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, cost);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battle_target(duel).is_some());
    e3.set_operation(duel, op);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) <= 2300 && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let sg = duel.get_matching_group(thfilter, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let sg = duel.get_matching_group(thfilter, a.tp, 0, LOCATION_MZONE, None);
    duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_LINK) && c.is_able_to_extra(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.has_non_zero_attack(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, None)
            && duel.is_existing_matching_card(cfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let g = duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel.get_matching_group(cfilter, a.tp, LOCATION_GRAVE, 0, None);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0 {
        let Some(tc) = duel.get_first_target() else { return Ok(()) };
        if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            e1.set_value(duel, 0);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = c.get_battle_target(duel) else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_battle(duel) {
        let atk = (f64::from(tc.get_attack(duel)) / 2.0).round() as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE_CAL);
        e1.set_value(duel, atk);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
