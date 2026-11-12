// Changer Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 25652655;

// On the field with Petit Angel (Level 3) and Armory Arm in the Extra Deck to Synchro Summon, and the opponent's Sangans to change position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (38142739, MZONE),
    (29071332, EXTRA),
    (29071332, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_can_change_position(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_POSITION, g, count as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}
