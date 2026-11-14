// Crystal Beast Emerald Tortoise
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 68215963;

// Copies and Gene-Warped Warwolves to attack and then change to Defense Position; the opponent's Raigeki to place it as a Spell.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (69247929, HAND),
    (69247929, HAND),
    (12580477, OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_redirect(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

// The Crystal Beasts' "send replace" effect.
pub fn add_redirect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT_CB);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_condition(duel, repcon);
    e1.set_operation(duel, repop);
    c.register_effect(duel, e1);
}

fn repcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel) && c.is_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_DESTROY)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    become_continuous_spell(duel, c, c);
    duel.raise_event(c, EVENT_CUSTOM + CARD_CRYSTAL_TREE, Some(a.e), 0, a.tp, 0, 0)
}

pub fn become_continuous_spell(duel: &mut Duel, owner: CardId, tc: CardId) {
    let e1 = duel.create_effect(owner);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
    e1.set_value(duel, (TYPE_SPELL + TYPE_CONTINUOUS) as i32);
    tc.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && c.get_attacked_count(duel) > 0 && c.is_can_change_position(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position_full(tc, [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, 0, 0], false)?;
    }
    Ok(())
}
