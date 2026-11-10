// Angel Statue - Azurune
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 44822037;

// Set and in the hand, with Summoned Skull on the opponent's side to destroy it in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (70781052, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_target(duel, bttg);
    e2.set_operation(duel, btop);
    c.register_effect(duel, e2);
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPE_MONSTER | TYPE_EFFECT, 1800, 1800, 4, RACE_FAIRY, ATTRIBUTE_LIGHT);
    duel.is_player_can_special_summon_monster(tp, ID, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon(duel, tp) {
        return Ok(());
    }
    c.add_monster_attribute(duel, TYPE_EFFECT + TYPE_TRAP);
    duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
    c.add_monster_attribute_complete(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_SPSUMMON);
    e1.set_condition(duel, |duel, a| duel.get_current_chain() == 0 && a.tp != a.ep);
    e1.set_cost(duel, ngcost);
    e1.set_target(duel, ngtg);
    e1.set_operation(duel, ngop);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect_full(duel, e1, true);
    duel.special_summon_complete()?;
    Ok(())
}

fn ngfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summon_location(duel, LOCATION_SZONE) && c.is_continuous_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn ngcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(ngfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, ngfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn ngtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn ngop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}

fn bttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    if chk == 0 {
        return Ok(bc.is_relate_to_battle(duel));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(bc.into()), 1, 0, 0);
    Ok(false)
}

fn btop(duel: &mut Duel, a: &Args) -> Script<()> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    unported!(duel, bc.is_none());
    if let Some(bc) = bc
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
