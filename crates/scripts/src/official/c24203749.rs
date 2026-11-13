// The Great Gallant Bandit
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 24203749;

// Copies and monsters in the hand to Tribute Summon with, the opponent's monsters to Tribute and to change to Defense Position, and
// its Pot of Greed in the hand to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (26202165, HAND),
    (26202165, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(2),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SUMMON_PROC);
    e0.set_condition(duel, sumcon);
    e0.set_target(duel, sumtg);
    e0.set_operation(duel, sumop);
    e0.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, discon);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_battle_phase());
    e2.set_cost(duel, revealcost);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    e2.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

// IsReleasable gets the player as its reason, as the Lua passes it through GetMatchingGroup.
fn tributes(duel: &Duel, tp: u8, c: CardId) -> CardSet {
    duel.get_matching_group(
        |mc, duel| mc.is_releasable_full(duel, u32::from(tp), None),
        tp,
        LOCATION_HAND,
        LOCATION_MZONE,
        Some(c.into()),
    )
}

fn tribute_counts(duel: &Duel, c: CardId, minc: u16) -> (i32, i32) {
    let (min_tributes, max_tributes) = c.get_tribute_requirement(duel);
    (min_tributes.max(i32::from(minc)), max_tributes)
}

fn sumcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let (min_tributes, max_tributes) = tribute_counts(duel, c, a.min);
    if min_tributes <= 0 || min_tributes > max_tributes {
        return false;
    }
    let tp = c.get_controler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 || duel.get_field_group_count(tp, LOCATION_MZONE, 0) > 0 {
        return false;
    }
    let mg = tributes(duel, tp, c);
    !mg.is_empty() && mg.len() as i32 >= min_tributes
}

fn sumtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let (min_tributes, max_tributes) = tribute_counts(duel, c, a.min);
    let mg = tributes(duel, tp, c);
    let cancelable = duel.is_summon_cancelable();
    let rescon = |_: &Duel, _: CardSet, _: CardSet, _: Option<CardId>| (true, false);
    let sg = aux::select_unselect_group_full(
        duel,
        mg,
        min_tributes.max(0) as usize,
        max_tributes.max(0) as usize,
        &rescon,
        tp,
        HINTMSG_RELEASE,
        None,
        cancelable,
    )?;
    if sg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let sg = a.e.get_label_object_group(duel).unwrap_or_default();
    if !sg.is_empty() {
        c.set_material(duel, Some(sg));
        duel.release(sg, REASON_SUMMON | REASON_MATERIAL)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    a.ep == 1 - a.tp
        && a.e.get_handler(duel).is_normal_summoned(duel)
        && chain::is_triggering_location(duel, ev, LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED)
        && duel.is_chain_disablable(ev)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_effect(a.ev as u8) && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re)) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    if chk == 0 {
        return Ok(!g.is_empty() && !g.is_exists(duel, CardId::is_public, 1, None));
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(posfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_POSITION, None, 1, 1 - tp, POS_FACEUP_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(posfilter, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
