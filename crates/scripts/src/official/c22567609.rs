// Nimble Momonga
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 22567609;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field and in the Deck, destroyed in battle by opponent's monsters (Cyber Falcon, a Normal Monster).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
// The face-down Special Summon after the prompt writes no MSG_SPSUMMONING, so no check counts it.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.get_reason(duel) & REASON_BATTLE != 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 1000);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 0, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ID) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.recover(tp, 1000, REASON_EFFECT)? > 0 {
        let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
        if ft <= 0 {
            return Ok(());
        }
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let g = duel.get_matching_group(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, ft as u8, None)?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
        }
    }
    Ok(())
}
