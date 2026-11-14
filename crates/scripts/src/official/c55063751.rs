// Gameciel, the Sea Turtle Kaiju
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 55063751;

pub const LISTED_SERIES: &[u16] = &[SET_KAIJU];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with a copy on the opponent's field to Tribute, Kyoutou Waterfront for Kaiju Counters, Raigeki and Dark Hole to fill it, and
// the opponent's Monster Reborn and Pot of Greed to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE | OPPONENT),
    (56111151, HAND),
    (12580477, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
    (83764718, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::add_kaiju_procedure(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, negcon);
    e3.set_cost(duel, aux::kaiju_counter_cost::<2>);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    if a.ep == a.tp || a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    a.re.is_some_and(|re| !re.get_handler(duel).is_code(duel, ID)) && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(
            rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || (!relation && duel.is_player_can_remove(tp, None))
        );
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if relation {
        let (controler, location) = (rc.get_controler(duel), rc.get_location(duel));
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(rc.into()), 1, controler, location as i32);
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, 0, rc.get_previous_location(duel) as i32);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
