// Archfiend Eccentrick
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 57624336;

// A copy in each zone, opponent's Spells/Traps to target and monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (5318639, SZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, destg1);
    e2.set_operation(duel, desop1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_tribute);
    e3.set_target(duel, destg2);
    e3.set_operation(duel, desop2);
    c.register_effect(duel, e3);
}

fn destg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(c.is_destructable(duel)
            && duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let mut g = duel
        .select_target(
            tp,
            CardId::is_spell_trap,
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            1,
            Some(c.into()),
        )?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, 0, 0);
    Ok(false)
}

fn desop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, e) {
        duel.destroy(CardSet::from_iter([tc, c]), REASON_EFFECT)?;
    }
    Ok(())
}

fn destg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_monster(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
