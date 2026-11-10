// Zalen the Shackled Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 4891376;

// On the field with two Sangan, Effect Monsters whose activations it can chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_NONTUNER);
    e1.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.get_current_chain() > 1);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

pub fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel);
    if g.is_exists(duel, |c, duel| c.is_type(duel, TYPE_TUNER), 2, None) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_MULTIPLE_TUNERS);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD) | RESET_PHASE | PHASE_END);
        c.register_effect(duel, e1);
    }
}

fn triggering_handler(duel: &Duel, ch: u8) -> Option<(EffectId, CardId)> {
    let eff = duel.get_chain_info::<Option<EffectId>>(ch, CHAININFO_TRIGGERING_EFFECT)?;
    Some((eff, eff.get_handler(duel)))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, ev) = (a.e, a.tp, a.ev as u8);
    let b1 = duel.is_chain_disablable(ev);
    let b2 = duel.is_chain_disablable(ev - 1);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    let ch = i32::from(ev) - op.map_or(0, |op| op as i32) + 1;
    e.set_label(duel, ch.into());
    let Some((eff, rc)) = triggering_handler(duel, ch as u8) else {
        return Ok(false);
    };
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(rc.into()), 1, tp, 0);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, eff) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(rc.into()), 1, tp, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ch = a.e.get_label(duel) as u8;
    let Some((eff, rc)) = triggering_handler(duel, ch) else {
        return Ok(());
    };
    if duel.negate_effect(ch) && rc.is_relate_to_effect(duel, eff) {
        duel.destroy(rc, REASON_EFFECT)?;
    }
    Ok(())
}
