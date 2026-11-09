// Star Ryzeal
use super::c8633261::extra_deck_lock;
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 84433129;

// In the hand, Special Summoned by detaching a material that Xyz Import attached to Dark Requiem Xyz Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1621413, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// No Ryzeal Spell or Trap is ported, so the Set effect has nothing to find.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.check_remove_overlay_card(tp, 1, 0, 1, REASON_COST)
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVEXYZ);
    let g = duel.get_overlay_group(tp, 1, 0).select(duel, tp, 0, 1, None)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(g) = e.get_label_object_group(duel) else { return Ok(()) };
    let Some(xyzc) = g.get_first().and_then(|mc| mc.get_overlay_target(duel)) else {
        return Ok(());
    };
    duel.send_to_grave(g, REASON_COST)?;
    duel.raise_single_event(xyzc, EVENT_DETACH_MATERIAL, Some(e), 0, 0, 0, 0)?;
    duel.raise_event(xyzc, EVENT_DETACH_MATERIAL, Some(e), REASON_EFFECT, tp, tp, 0)?;
    if let Some(c) = a.c {
        extra_deck_lock(duel, c, tp, aux::stringid(ID, 2));
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
