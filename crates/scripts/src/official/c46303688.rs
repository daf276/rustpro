// Roulette Barrel
use engine::constants::MSG_TOSS_DICE;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 46303688;

// Monsters of several Levels on both fields to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (1929294, MZONE),
    (6367785, MZONE | OPPONENT),
    (2468169, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_TOSS_DICE)];

// s.roll_dice, which only cards that are not ported read, is left out.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DICE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, a.tp, 2);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let &[d1, d2] = duel.toss_dice(tp, 2)?.as_slice() else {
        return Ok(());
    };
    let (low, high) = (u64::from(d1.min(d2)), u64::from(d1.max(d2)));
    let sel = if low == high {
        u64::from(d1)
    } else {
        duel.announce_number(tp, &[low, high])?
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(
        tp,
        |c, duel| c.is_faceup(duel) && u64::from(c.get_level(duel)) == sel,
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    if !dg.is_empty() {
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
