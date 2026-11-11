// Shade the Obscure
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, PZONE};

const ID: u32 = 77157846;

// A copy in the Pendulum Zone to return, in the hand to Special Summon next to Flash Knight in the other scale, and on the field
// to pay for destroying a Pendulum Monster and adding Flash Knight from the Deck; Sangan to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, MZONE),
    (17390179, PZONE),
    (17390179, HAND),
    (17390179, DECK),
    (17390179, DECK),
    (26202165, HAND),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        !c.is_pendulum_monster(duel) && sumtype & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, aux::faceup_filter(|c, duel| c.is_attack_below(duel, 1000)), 1, None))
    });
    e2.set_target(duel, rthtg);
    e2.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_PZONE, 0) > 0);
    e3.set_target(duel, stock::special_summon_self_target_player_0);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOEXTRA);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_cost(duel, Cost::pay_lp::<1000>);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_original_type(duel, TYPE_PENDULUM) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if tc.is_on_field(duel) {
        duel.hint_selection(g);
    }
    if duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let dg = duel.get_matching_group(
        |c, duel| c.is_pendulum_monster(duel) && c.is_able_to_extra(duel),
        tp,
        LOCATION_DECK,
        0,
        None,
    );
    if dg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
    let sg = dg.select(duel, tp, 1, 1, None)?;
    if !sg.is_empty() {
        duel.break_effect()?;
        duel.send_to_extra_p(sg, tp, REASON_EFFECT)?;
    }
    Ok(())
}
