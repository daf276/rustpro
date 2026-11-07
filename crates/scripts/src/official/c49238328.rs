// Pot of Extravagance
use crate::prelude::*;

const ID: u32 = 49238328;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1) && !duel.check_phase_activity());
    e1.set_cost(duel, |duel, a, chk| {
        a.e.set_label(duel, 100);
        Ok(chk == 0)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(cfilter, tp, LOCATION_EXTRA, 0, None);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_player_can_draw(tp, 1) && g.len() >= 3);
    }
    let op = if duel.is_player_can_draw(tp, 2) && g.len() >= 6 {
        duel.select_option(tp, &[aux::stringid(ID, 0), aux::stringid(ID, 1)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 0)])?
    };
    let rg = g.random_select(duel, tp, 3 + op as usize * 3);
    duel.remove(rg, POS_FACEDOWN, REASON_COST)?;
    duel.set_target_player(tp);
    duel.set_target_param(op as u32 + 1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, op + 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
