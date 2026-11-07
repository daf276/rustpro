// Forbidden Droplet
use crate::prelude::*;

const ID: u32 = 24299458;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, 100);
        Ok(true)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn effect_monster(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ct = duel.get_matching_group_count(effect_monster, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(ct > 0
            && duel.is_existing_matching_card(
                CardId::is_able_to_grave_as_cost,
                tp,
                LOCATION_ONFIELD | LOCATION_HAND,
                0,
                1,
                Some(c.into()),
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        CardId::is_able_to_grave_as_cost,
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        ct as u8,
        Some(c.into()),
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    let og = duel.get_operated_group();
    e.set_label(duel, og.len() as i64);
    let typ = og.iter().fold(0, |typ, tc| typ | tc.get_original_type(duel));
    duel.set_chain_limit(chainlimit, i64::from(typ & (TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP)));
    duel.set_operation_info(0, CATEGORY_DISABLE, None, og.len() as u8, 0, 0);
    Ok(false)
}

fn chainlimit(duel: &Duel, e: EffectId, rp: u8, tp: u8, typ: i64) -> bool {
    tp == rp || i64::from(e.get_handler(duel).get_original_type(duel)) & typ == 0
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = e.get_label(duel) as u8;
    let sg = duel.get_matching_group(effect_monster, tp, 0, LOCATION_MZONE, None);
    if sg.len() < usize::from(ct) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = sg.select(duel, tp, ct, ct, None)?;
    duel.hint_selection(g);
    let c = e.get_handler(duel);
    for tc in g {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e0 = duel.create_effect(c);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e0.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e0.set_value(duel, (tc.get_attack(duel) + 1) / 2);
        tc.register_effect(duel, e0);
        super::c97268402::negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
    }
    Ok(())
}
