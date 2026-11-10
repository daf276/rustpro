// Time Wizard
// s.toss_coin, which only cards that are not ported read, is left out.
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 71625222;

// Monsters on both sides to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_COIN + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.call_coin(tp)? {
        let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
        duel.destroy(g, REASON_EFFECT)?;
        duel.raise_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, 0, tp, 0)?;
    } else {
        let g = duel.get_field_group(tp, LOCATION_MZONE, 0);
        duel.destroy(g, REASON_EFFECT)?;
        let dg = duel
            .get_operated_group()
            .filter(duel, |c, duel| c.is_previous_position(duel, POS_FACEUP), None);
        let sum = dg.get_sum(duel, |c, duel| i64::from(c.get_previous_attack_on_field(duel)));
        if sum > 0 {
            duel.damage(tp, (sum as f64 / 2.0).round() as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
