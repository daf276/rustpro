// Freed the Brave Wanderer
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 16556849;

// Key Mace and White Duston, LIGHT, in the Graveyard and on the field to banish, with a high-ATK monster of the opponent to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (1929294, GRAVE),
    (3557275, GRAVE),
    (1929294, MZONE),
    (3557275, MZONE),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let location = LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, location, 0, 2, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, costfilter, tp, location, 0, 2, 2, Some(c.into()))?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) > atk
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let atk = c.get_attack(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, atk));
    }
    let filter = |c, duel: &Duel| filter(c, duel, atk);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, a.e)
        && c.is_faceup(duel)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
        && tc.get_attack(duel) > c.get_attack(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
