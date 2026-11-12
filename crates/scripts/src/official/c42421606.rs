// Number 85: Crazy Box
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 42421606;
pub const XYZ_NUMBER: u32 = 85;

// In the Extra Deck, Xyz Summoned with two Level 4 monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (2118022, MZONE), (2483611, MZONE), (4042268, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// s.roll_dice, which only cards that are not ported read, is left out.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DICE + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, eftg);
    e2.set_operation(duel, efop);
    c.register_effect(duel, e2);
}

fn eftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, a.tp, 1);
    Ok(false)
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(&dc) = duel.toss_dice(tp, 1)?.first() else { return Ok(()) };
    match dc {
        1 => {
            let lp = duel.get_lp(tp);
            // Lua halves with a float, which the core rounds.
            duel.set_lp(tp, (f64::from(lp) / 2.0).round() as i32);
        }
        2 => {
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
        3 => {
            if duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0 {
                duel.discard_hand(1 - tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
            }
        }
        4 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
            let g = duel.select_matching_card(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
            if let Some(tc) = g.get_first() {
                duel.hint_selection(g);
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e1.set_code(duel, EFFECT_DISABLE);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                tc.register_effect(duel, e1);
                let e2 = duel.create_effect(c);
                e2.set_type(duel, EFFECT_TYPE_SINGLE);
                e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e2.set_code(duel, EFFECT_DISABLE_EFFECT);
                e2.set_value(duel, RESET_TURN_SET as i32);
                e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
                tc.register_effect(duel, e2);
            }
        }
        5 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        _ => {
            if c.is_relate_to_effect(duel, e) {
                duel.destroy(c, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
