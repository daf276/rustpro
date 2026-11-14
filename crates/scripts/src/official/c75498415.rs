// Blackwing - Sirocco the Dawn
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 75498415;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// In the hand to Summon without Tributes against the opponent's Sangan, then to add the ATK of its other copies.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_PROC);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1));
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    a.min == 0
        && c.get_level(duel) > 4
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BLACKWING])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let e2 = duel.create_effect(a.e.get_handler(duel));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_OATH);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, e, c| i64::from(c.get_field_id(duel)) != e.get_label(duel));
    let fid = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_field_id(duel));
    e2.set_label(duel, i64::from(fid));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) {
        let atk: i32 = duel
            .get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, Some(tc.into()))
            .into_iter()
            .map(|bc| bc.get_attack(duel))
            .sum();
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, atk);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
