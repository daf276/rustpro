// Bottomless Trap Hole
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_target(duel, target2);
    e3.set_operation(duel, activate2);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, tp: u8, ep: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel) && c.get_attack(duel) >= 1500 && ep != tp && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(eg) = a.eg else { return Ok(false) };
    let Some(tc) = eg.get_first() else { return Ok(false) };
    if chk == 0 {
        return Ok(filter(tc, duel, a.tp, a.ep));
    }
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tc.into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) && tc.get_attack(duel) >= 1500 {
        duel.destroy_full(tc, REASON_EFFECT, LOCATION_REMOVED, duel.get_reason_player())?;
    }
    Ok(())
}

fn filter2(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
        && c.get_attack(duel) >= 1500
        && c.get_summon_player(duel) != tp
        && c.is_able_to_remove(duel)
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, Some(eg)) = (a.tp, a.eg) else { return Ok(false) };
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| filter2(c, duel, tp), 1, None));
    }
    let g = eg.filter(duel, |c, duel| filter2(c, duel, tp), None);
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn filter3(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.get_attack(duel) >= 1500
        && !c.is_summon_player(duel, tp)
        && c.is_relate_to_effect(duel, e)
        && c.is_location(duel, LOCATION_MZONE)
}

fn activate2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(eg) = a.eg else { return Ok(()) };
    let g = eg.filter(duel, |c, duel| filter3(c, duel, e, tp), None);
    if !g.is_empty() {
        duel.destroy_full(g, REASON_EFFECT, LOCATION_REMOVED, duel.get_reason_player())?;
    }
    Ok(())
}
