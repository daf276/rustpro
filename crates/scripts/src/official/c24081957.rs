// Sinful Spoils of Subversion - Snake-Eye
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 24081957;

// In the hand with monsters of both players on the field (Sangan) to place in a Spell and Trap Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8, hand_chk: bool) -> bool {
    if c.is_facedown(duel) {
        return false;
    }
    let owner = c.get_owner(duel);
    let ft = duel.get_location_count(owner, LOCATION_SZONE) - i32::from(owner == tp && hand_chk);
    ft > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let hand_chk = a.e.get_handler(duel).is_location(duel, LOCATION_HAND);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel, tp, hand_chk));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c, duel| tgfilter(c, duel, tp, hand_chk),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(
        tp,
        |c, duel| tgfilter(c, duel, tp, hand_chk),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e) && !tc.is_immune_to_effect(duel, a.e)) {
        return Ok(());
    }
    let owner = tc.get_owner(duel);
    if duel.get_location_count(owner, LOCATION_SZONE) == 0 {
        duel.send_to_grave_full(tc, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
    } else if duel.move_to_field(tc, tp, owner, LOCATION_SZONE, POS_FACEUP, tc.is_monster_card(duel))? {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
        tc.register_effect(duel, e1);
    }
    Ok(())
}
