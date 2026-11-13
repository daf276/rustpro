// Varar, Vaalmonican Concord
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 1340142;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| matfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

// On the field with Angello and Dimonno in the Pendulum Zone, and the opponent's Special Summons to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (3048768, PZONE),
    (30432463, PZONE),
    (65496951, HAND),
    (42193638, HAND),
    (5605529, HAND),
    (78598237, HAND),
    (2815176, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, EXTRA | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.get_counter(a.e.get_handler_player(duel), 1, 0, COUNTER_RESONANCE) >= 6
    });
    e1.set_value(duel, Value::Effect(|duel, _, te| !te.is_card_setcode(duel, SET_VAALMONICA)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| duel.get_matching_group_count(atkfilter, e.get_handler_player(duel), LOCATION_MZONE, 0, None) as i32),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_SPSUMMON);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e3.set_target(duel, negsumtg);
    e3.set_operation(duel, negsumop);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_VAALMONICA], Some(lc), sumtype, tp) && c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4]) && c.is_set_card(duel, &[SET_VAALMONICA]) && c.is_faceup(duel)
}

fn negsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_can_remove_counter(tp, 1, 0, COUNTER_RESONANCE, 3, REASON_EFFECT));
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn negsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (eg, tp) = (a.eg.unwrap_or_default(), a.tp);
    duel.negate_summon(eg);
    if duel.destroy(eg, REASON_EFFECT)? > 0 {
        duel.break_effect()?;
        duel.remove_counter(tp, 1, 0, COUNTER_RESONANCE, 3, REASON_EFFECT)?;
    }
    Ok(())
}
