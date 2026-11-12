// Medicurius the Power Patron of Illusions
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 4063756;

// In the Extra Deck, Link Summoned with Purification Power Patron (an Extra Deck monster) and Dark Magician (Normal Monsters), with a
// monster of the opponent to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (31822037, MZONE),
    (31822037, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_EXTRA, Some(lc), sumtype, tp), 1, None)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, commoncondition::<1>);
    e1.set_target(duel, disableatktg);
    e1.set_operation(duel, disableatkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, commoncondition::<2>);
    e2.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner_player(duel) == 1 - e.get_handler_player(duel) && te.is_activated(duel)),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && a.e.get_handler(duel).get_linked_group_count(duel) == 3
    });
    e3.set_target(duel, bantg);
    e3.set_operation(duel, banop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn commoncondition<const MINIMUM_NUMBER: usize>(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_linked_group_count(duel) >= MINIMUM_NUMBER
}

fn disableatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn disableatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, 0, false);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        // The core rounds the Lua's `atk/2` where it reads it as an integer.
        e1.set_value(duel, (tc.get_attack(duel) + 1) / 2);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let mut g = duel.get_matching_group(CardId::is_able_to_remove, a.tp, 0, LOCATION_ONFIELD, None);
    if c.is_relate_to_effect(duel, a.e) && c.is_able_to_remove(duel) {
        g.insert(c);
    }
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
