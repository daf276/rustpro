// Don Zaloog
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 76922029;

// Two on the field, and the opponent's Sangan to deal battle damage to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0;
    let b2 = duel.is_player_can_discard_deck(1 - tp, 2);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_HANDES);
        duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 1);
    } else {
        e.set_category(duel, CATEGORY_DECKDES);
        duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, 1 - tp, 2);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.e.get_label(duel) == 1 {
        let g = duel.get_field_group(tp, 0, LOCATION_HAND);
        if g.is_empty() {
            return Ok(());
        }
        let sg = g.random_select(duel, tp, 1);
        duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    } else {
        duel.discard_deck(1 - tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}
