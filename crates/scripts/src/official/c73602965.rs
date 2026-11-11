// Arias the Labrynth Butler
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 73602965;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and the GY, with a Labrynth monster (Arianna) and Normal Traps (Trap Hole) in the hand, a Set Trap to chain to, and
// the opponent's Raigeki and Solemn Judgment to respond.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (1225009, HAND),
    (4206964, HAND),
    (4206964, SZONE),
    (44095762, SZONE),
    (12580477, HAND | OPPONENT),
    (41420027, HAND | OPPONENT),
    (1225009, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, spsettarget);
    e1.set_operation(duel, spsetop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfspcon);
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn spsetfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    (ft > 0 && c.is_set_card(duel, &[SET_LABRYNTH]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
        || (c.is_normal_trap(duel) && c.is_ssetable(duel))
}

fn spsettarget(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ft = duel.get_mzone_count(tp, Some(c.into()));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|g, duel| spsetfilter(g, duel, e, tp, ft), tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spsetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, |c, duel| spsetfilter(c, duel, e, tp, ft), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sc.is_monster(duel) {
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    } else if sc.is_trap(duel) && duel.sset(tp, sc)? > 0 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    Ok(())
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let ch = a.ev as u8 - 1;
    if ch == 0 || a.ep == a.tp {
        return false;
    }
    let ch_player: u8 = duel.get_chain_info(ch, CHAININFO_TRIGGERING_PLAYER);
    let Some(ch_eff) = duel.get_chain_info::<Option<EffectId>>(ch, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let ch_c = ch_eff.get_handler(duel);
    ch_player == a.tp
        && ((ch_c.is_set_card(duel, &[SET_LABRYNTH]) && !ch_c.is_code(duel, ID))
            || (ch_c.get_original_type(duel) == TYPE_TRAP && ch_eff.is_trap_effect(duel)))
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}
