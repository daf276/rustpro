// Tytannial, Princess of Camellias
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 11819616;

// On the field, with the opponent's Mage Power and Mind Control in hand to target it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (83746708, HAND | OPPONENT), (37520316, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, discost);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, CardId::is_on_field, 1, None)) && duel.is_chain_negatable(a.ev as u8)
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PLANT) && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, costfilter, (1, 1), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, costfilter, (1, 1), false, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
