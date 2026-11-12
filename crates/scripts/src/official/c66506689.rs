// Tin Archduke
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 66506689;

// In the Extra Deck, Xyz Summoned with three Level 4 Normal Monsters (Prevent Rat, Kojikocy, Uraby), with a monster on the
// opponent's side to change the position of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (1784619, MZONE),
    (549481, MZONE),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<4, 3>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_count_limit(duel, 1, 0);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_change_position(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            true,
        )?;
    }
    Ok(())
}
