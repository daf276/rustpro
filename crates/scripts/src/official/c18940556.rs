// Ultimate Conductor Tyranno
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 18940556;

// In the hand, Special Summoned by banishing two Dinosaurs (Uraby), with the opponent's monsters to flip and fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1784619, GRAVE),
    (1784619, GRAVE),
    (1784619, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, sprcon);
    e1.set_target(duel, sprtg);
    e1.set_operation(duel, sprop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_ATTACK_ALL);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DAMAGE + CATEGORY_TOGRAVE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BATTLE_START);
    e4.set_target(duel, tgtg);
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
}

fn sprfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn sprcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    duel.get_location_count(tp, LOCATION_MZONE) > -2
        && rg.len() > 1
        && aux::select_unselect_group_check(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1))
}

fn sprtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(sprfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let g = aux::select_unselect_group(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn sprop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(())
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_public(duel) || c.is_monster(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_monster, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(posfilter, tp, 0, LOCATION_MZONE, 1, None),
        );
    }
    let dg = duel.get_matching_group(CardId::is_monster, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    let g = duel.get_matching_group(posfilter, tp, 0, LOCATION_MZONE, None);
    let destroyed = if duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None) {
        None
    } else {
        Some(dg)
    };
    duel.set_operation_info(0, CATEGORY_DESTROY, destroyed, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_matching_card(tp, CardId::is_monster, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if !g1.is_empty() && duel.destroy(g1, REASON_EFFECT)? != 0 {
        let g2 = duel.get_matching_group(posfilter, tp, 0, LOCATION_MZONE, None);
        if !g2.is_empty() {
            duel.change_position(g2, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let d = duel.get_attack_target();
    if chk == 0 {
        return Ok(duel.get_attacker() == Some(a.e.get_handler(duel)) && d.is_some_and(|d| d.is_defense_pos(duel)));
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, d.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.damage(1 - a.tp, 1000, REASON_EFFECT)? != 0
        && let Some(d) = duel.get_attack_target()
        && d.is_relate_to_battle(duel)
        && d.is_defense_pos(duel)
    {
        duel.send_to_grave(d, REASON_EFFECT)?;
    }
    Ok(())
}
