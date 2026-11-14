// Tempura of Fortune - EBI
use crate::prelude::*;
use crate::testing::{Check, PZONE};

const ID: u32 = 86304179;

pub const TOSS_COIN: bool = true;

// In the Pendulum Zone, with a copy to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (ID, PZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COIN + CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, cointg);
    e1.set_operation(duel, coinop);
    c.register_effect(duel, e1);
}

fn cointg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && chkc.is_controler(duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_PZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_PZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn coinop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let res = duel.toss_coin(tp, 1)?.first().map(|&res| u32::from(res));
        if res == Some(COIN_HEADS) {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        } else if res == Some(COIN_TAILS) && duel.destroy(tc, REASON_EFFECT)? > 0 {
            duel.set_lp(tp, duel.get_lp(tp) - tc.get_scale(duel) as i32 * 300);
        }
    }
    Ok(())
}
