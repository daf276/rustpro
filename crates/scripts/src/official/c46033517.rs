// Machina Ruinforce
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 46033517;

pub const LISTED_SERIES: &[u16] = &[SET_MACHINA];
// In the Graveyard to Special Summon itself by banishing two Level 7 Machines, and on the field against Mirror Force in the BP.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, MZONE),
    (3797883, GRAVE),
    (87322377, GRAVE),
    (87322377, MZONE),
    (44095762, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
];
// The Machina monsters in banishment need an unported Machina card.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, |_, a| a.r & (REASON_EFFECT | REASON_BATTLE) != 0);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel) && c.is_monster(duel) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_remove_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    let g = duel.get_matching_group(spfilter, tp, LOCATION_GRAVE, 0, Some(a.e.get_handler(duel).into()));
    g.check_with_sum_greater(duel, |c, duel| c.get_level(duel), 12)
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let rg = duel.get_matching_group(spfilter, a.tp, LOCATION_GRAVE, 0, Some(a.e.get_handler(duel).into()));
    let g = rg.select_with_sum_greater(duel, a.tp, |c, duel| c.get_level(duel), 12)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp != a.tp
        && duel.is_battle_phase()
        && duel.is_chain_negatable(a.ev as u8)
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.pay_lp_cost(a.tp, (duel.get_lp(a.tp) / 2) as u32)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if !duel.negate_activation(a.ev as u8) {
        return Ok(());
    }
    if re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    duel.set_lp(1 - a.tp, duel.get_lp(1 - a.tp) / 2);
    Ok(())
}

fn rescon2(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.get_sum(duel, |c, duel| i64::from(c.get_level(duel))) <= 12, false)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MACHINA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter2(c, duel, a.e, a.tp), LOCATION_REMOVED)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut lc = duel.get_location_count(tp, LOCATION_MZONE);
    if lc <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        lc = 1;
    }
    let rg = duel.get_matching_group(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_REMOVED, 0, None);
    let g = aux::select_unselect_group(duel, rg, 1, lc.min(3) as usize, &rescon2, tp, HINTMSG_SPSUMMON)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
