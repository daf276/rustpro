// Zolga
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 16268841;

// On the field, as the Tribute for Level 5 Normal Monsters (Orion the Battle King, Shapesnatch) Tribute Summoned from the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (2971090, HAND), (2971090, HAND), (4035199, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_RELEASE);
    e1.set_condition_mut(duel, reccon);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
}

fn reccon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let previous = c.get_previous_controler(duel);
    a.e.set_label(duel, i64::from(previous));
    c.is_reason(duel, REASON_SUMMON)
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_event(EVENT_SUMMON_SUCCESS) || duel.check_event(EVENT_MSET));
    }
    let label = a.e.get_label(duel) as u8;
    duel.set_target_player(label);
    duel.set_target_param(2000);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, label, 2000);
    Ok(false)
}

fn recop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
