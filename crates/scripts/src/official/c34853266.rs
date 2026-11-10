// Tsukuyomi
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 34853266;

// In the hand to be Normal Summoned, with monsters on both fields to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (14851496, MZONE | OPPONENT),
    (4042268, MZONE | OPPONENT),
    (5628232, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Spirit::add_procedure(duel, c, &[EVENT_SUMMON_SUCCESS, EVENT_FLIP]);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_FLIP);
    c.register_effect(duel, e3);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(
        0,
        CATEGORY_POSITION,
        g,
        g.map_or(0, |g| g.len()) as u8,
        tp,
        POS_FACEDOWN_DEFENSE as i32,
    );
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
