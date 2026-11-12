// Sunny Pixie
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 73837870;

// On the field with Swordsman of Landstar (Level 3) for a Level 4 Synchro Summon of Poplar of the White Forest (LIGHT).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (5800323, EXTRA),
    (5800323, EXTRA),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e1.set_condition(duel, reccon);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
}

fn reccon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && a.r == REASON_SYNCHRO
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_LIGHT))
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(a.tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, 1000);
    Ok(false)
}

fn recop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
