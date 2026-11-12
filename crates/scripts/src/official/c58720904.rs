// Pendransaction
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 58720904;

// In the Extra Deck with copies to outnumber the opponent's Extra Deck, Xyz Summoned with two Level 4 Normal Monsters (Prevent Rat).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn extra_deck_difference(duel: &Duel, tp: u8) -> i32 {
    duel.get_field_group_count(tp, LOCATION_EXTRA, 0) as i32 - duel.get_field_group_count(tp, 0, LOCATION_EXTRA) as i32
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && extra_deck_difference(duel, a.tp) > 0)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let val = extra_deck_difference(duel, tp);
    if val >= 1 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        e1.set_value(duel, 1000);
        c.register_effect(duel, e1);
    }
    if val >= 5 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        duel.break_effect()?;
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        e2.set_value(duel, 1);
        c.register_effect(duel, e2);
    }
    if val >= 10 {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    if val >= 15 {
        duel.break_effect()?;
        duel.set_lp(1 - tp, 3000);
    }
    Ok(())
}
