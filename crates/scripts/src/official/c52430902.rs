// Psychic Jumper
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 52430902;

// On the field with another Psychic (Stinger the Electric Expeller), with monsters on the opponent's side (Elfnote Lucina) to swap.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (57649113, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PSYCHIC) && c.get_code(duel) != ID && c.is_able_to_change_controler(duel)
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_change_controler(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter2, tp, 0, LOCATION_MZONE, 1, None)
            && duel.is_existing_target(filter1, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let mut g1 = duel
        .select_target(tp, filter1, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g2 = duel
        .select_target(tp, filter2, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g1.merge(g2);
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(g1), 2, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let mut cards = g.iter();
    let (tc1, tc2) = (cards.next(), cards.next());
    unported!(duel, tc1.is_none() || tc2.is_none());
    let (Some(tc1), Some(tc2)) = (tc1, tc2) else { return Ok(()) };
    if tc1.is_faceup(duel)
        && tc1.is_relate_to_effect(duel, a.e)
        && tc2.is_faceup(duel)
        && tc2.is_relate_to_effect(duel, a.e)
        && duel.swap_control(tc1, tc2, 0, 0)?
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc1.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        tc2.register_effect(duel, e2);
    }
    Ok(())
}
