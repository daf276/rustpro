// Raiza the Storm Monarch
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 73125233;

// In the hand, to be Tribute Summoned with Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (13597785, MZONE)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_tribute_summoned(duel));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first())
        && tc.is_able_to_deck(duel)
    {
        duel.set_operation_info(0, CATEGORY_TODECK, Some(tc.into()), 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    }
    Ok(())
}
