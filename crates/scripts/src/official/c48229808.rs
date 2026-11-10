// Horus the Black Flame Dragon LV8
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 48229808;

pub const LISTED_NAMES: &[u32] = &[11224103];

// Raigeki and Dark Hole in the opponent's hand to negate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (12580477, HAND | OPPONENT), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re
            .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_spell_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(re) = a.re else { return Ok(false) };
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let rc = re.get_handler(duel);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, Some(re)) = (a.e, a.re) else { return Ok(()) };
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
