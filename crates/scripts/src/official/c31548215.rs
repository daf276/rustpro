// Gravedigger's Trap Hole
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 31548215;

// Set on the field, against the opponent's Ash Blossom & Joyous Spring and Ghost Ogre & Snow Rabbit in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
    (59438930, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let activate_location: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.ep != a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_disablable(a.ev as u8)
        && [LOCATION_GRAVE, LOCATION_HAND, LOCATION_REMOVED].contains(&activate_location)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.re.is_some_and(|re| !re.get_handler(duel).is_status(duel, STATUS_DISABLED)));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(2000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 2000);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_effect(a.ev as u8) {
        duel.damage(1 - a.tp, 2000, REASON_EFFECT)?;
    }
    Ok(())
}
