// Link Apple
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 7925734;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_public(duel));
    }
    duel.confirm_cards(1 - tp, c)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_EXTRA, 0, 1, None)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(cfilter, tp, LOCATION_EXTRA, 0, None);
    let Some(rc) = g.random_select(duel, tp, 1).get_first() else {
        return Ok(());
    };
    if duel.remove(rc, POS_FACEUP, REASON_EFFECT)? == 0 || !rc.is_location(duel, LOCATION_REMOVED) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.break_effect()?;
    if rc.is_link_monster(duel)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    } else if duel.send_to_grave(c, REASON_EFFECT | REASON_DISCARD)? > 0 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
