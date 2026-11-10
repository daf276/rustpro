// Caius the Shadow Monarch
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 9748752;

// In the hand twice, to be Tribute Summoned with Shapesnatch, a DARK Normal Monster, which is also on the opponent's side to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (4035199, MZONE), (4035199, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
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
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first())
        && tc.is_able_to_remove(duel)
    {
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(tc.into()), 1, 0, 0);
        if tc.is_faceup(duel) && tc.is_attribute(duel, ATTRIBUTE_DARK) {
            duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
        }
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
        if tc.is_location(duel, LOCATION_REMOVED) && tc.is_monster(duel) && tc.is_attribute(duel, ATTRIBUTE_DARK) {
            duel.damage(1 - tp, 1000, REASON_EFFECT)?;
        }
    }
    Ok(())
}
