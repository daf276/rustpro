// Sillva, Warlord of Dark World
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 32619583;

// In the hand, to be discarded by Dark World Dealings from either side.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74117290, HAND), (74117290, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition_mut(duel, super::c34230233::descon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && a.rp != tp
        && i64::from(tp) == e.get_label(duel)
        && duel.get_field_group_count(tp, 0, LOCATION_HAND) > 1
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TODECK);
        let g = duel.select_matching_card(1 - tp, aux::TRUE, tp, 0, LOCATION_HAND, 2, 2, None)?;
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}
