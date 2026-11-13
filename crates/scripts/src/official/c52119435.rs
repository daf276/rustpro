// Geonator Transverser
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 52119435;

// In the Extra Deck, Link Summoned from Sangans, with the opponent's Sangan for it to point at and swap control with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e1.set_target_card(duel, indtg);
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_linked_group_count(duel) == 2);
    e2.set_target(duel, controltg);
    e2.set_operation(duel, controlop);
    c.register_effect(duel, e2);
}

fn indtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let oc = e.get_handler(duel);
    c == oc || oc.get_linked_group(duel).is_contains(c)
}

fn controltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e.get_handler(duel).get_linked_group(duel);
    if chk == 0 {
        return Ok(g.filter_count(duel, CardId::is_able_to_change_controler, None) == 2);
    }
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(g), 2, tp, 0);
    Ok(false)
}

fn controlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    let mut cards = g.into_iter();
    if let (Some(c1), Some(c2), 2) = (cards.next(), cards.next(), g.len())
        && g.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) != 1
    {
        duel.swap_control(c1, c2, 0, 0)?;
    }
    Ok(())
}
