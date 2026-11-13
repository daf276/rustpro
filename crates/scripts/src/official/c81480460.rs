// Barrel Dragon
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 81480460;

pub const TOSS_COIN: bool = true;

// Two copies on the field, with the opponent's Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_COIN + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 3);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let results = duel.toss_coin(tp, 3)?;
        if duel.count_heads(&results) >= 2 {
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}
