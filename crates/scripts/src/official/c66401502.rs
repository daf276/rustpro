// Vanquish Soul Pantera
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 66401502;

// In the hand and on the field, with EARTH (Hollie Sue) and FIRE (Razen) monsters in the hand to reveal, and Spells/Traps on
// both fields to destroy in its column.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (93156774, HAND),
    (29302858, HAND),
    (53129443, SZONE),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MMZONE, 0) == 0);
    e1.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e2.set_target(duel, vstg);
    e2.set_operation(duel, vsop);
    c.register_effect(duel, e2);
}

fn vscostfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_attribute(duel, att) && !c.is_public(duel)
}

fn vsrescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let count = |att| sg.filter_count(duel, |c, duel| c.is_attribute(duel, att), None) > 0;
    (count(ATTRIBUTE_EARTH) && count(ATTRIBUTE_FIRE), false)
}

fn column_spell_traps(duel: &Duel, c: CardId) -> CardSet {
    let mut colg = c.get_column_group(duel);
    colg.keep_matching(duel, |c, duel| c.is_spell_trap(duel), None);
    colg
}

fn vstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cg1 = duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_EARTH), tp, LOCATION_HAND, 0, None);
    let b1 = !cg1.is_empty() && (duel.is_able_to_enter_bp() || duel.is_battle_phase());
    let cg2 = cg1.union(duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_FIRE), tp, LOCATION_HAND, 0, None));
    let colg = column_spell_traps(duel, e.get_handler(duel));
    let b2 = !colg.is_empty() && aux::select_unselect_group_check(duel, cg2, 1, 2, &vsrescon);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let g = cg1.select(duel, tp, 1, 1, None)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            e.set_category(duel, 0);
        }
        Some(2) => {
            let g = aux::select_unselect_group_full(duel, cg2, 1, 2, &vsrescon, tp, HINTMSG_CONFIRM, Some(&vsrescon), false)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            e.set_category(duel, CATEGORY_DESTROY);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(colg), colg.len() as u8, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn vsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    match a.e.get_label(duel) {
        1 => {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3000);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            c.register_effect(duel, e1);
        }
        2 => {
            let mut colg = column_spell_traps(duel, c);
            if c.is_spell_trap(duel) {
                colg.insert(c);
            }
            if !colg.is_empty() {
                duel.destroy(colg, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
