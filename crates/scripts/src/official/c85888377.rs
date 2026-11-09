// Apophis the Swamp Deity
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 85888377;

// Set and in the hand, with Call of the Haunted, a Continuous Trap, face-up beside it and the opponent's monsters to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (97077563, SZONE),
    (97077563, SZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPE_MONSTER | TYPE_NORMAL, 2000, 2200, 6, RACE_REPTILE, ATTRIBUTE_EARTH);
    duel.is_player_can_special_summon_monster(tp, ID, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 0, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_trap(duel) && c.is_type(duel, TYPE_CONTINUOUS)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon(duel, tp) {
        return Ok(());
    }
    c.add_monster_attribute(duel, TYPE_NORMAL + TYPE_TRAP);
    duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
    c.add_monster_attribute_complete(duel);
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_ONFIELD, 0, Some(c.into()));
    let cg = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    if ct > 0 && !cg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let sg = cg.select(duel, tp, 1, ct as u8, None)?;
        if sg.is_empty() {
            return Ok(());
        }
        duel.hint_selection_full(sg, true);
        duel.break_effect()?;
        for tc in sg {
            duel.negate_related_chain(tc, RESET_TURN_SET);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_value(duel, RESET_TURN_SET as i32);
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e2);
            if tc.is_type(duel, TYPE_TRAPMONSTER) {
                let e3 = duel.create_effect(c);
                e3.set_type(duel, EFFECT_TYPE_SINGLE);
                e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
                e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
                tc.register_effect(duel, e3);
            }
        }
    }
    Ok(())
}
