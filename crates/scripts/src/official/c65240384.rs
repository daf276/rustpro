// Big Shield Gardna
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 65240384;

// Copies on the field, usually Set, with the opponent's Mind Control in hand to target one (Mage Power only targets face-up monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (37520316, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_NEGATE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_F);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL + EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !(re.is_spell_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) {
        return false;
    }
    let c = a.e.get_handler(duel);
    let tg = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.len() == 1 && tg.get_first() == Some(c)) && c.is_facedown(duel)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(c.into()), 1, a.tp, POS_FACEUP_DEFENSE as i32);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    // Duel.ChangePosition returns a count, and 0 is truthy in Lua.
    if c.is_relate_to_effect(duel, a.e) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
        duel.negate_activation(a.ev as u8);
    }
    Ok(())
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if Some(c) == duel.get_attack_target() && c.is_defense_pos(duel) && c.is_relate_to_battle(duel) {
        duel.change_position(c, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}
