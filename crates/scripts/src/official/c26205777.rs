// Thestalos the Firestorm Monarch
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 26205777;

// In the hand, to be Tribute Summoned with Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (13597785, MZONE)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_tribute_summoned(duel));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if !g.is_empty() {
        let sg = g.random_select(duel, 1 - tp, 1);
        duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
        if let Some(tc) = sg.get_first()
            && tc.is_monster(duel)
        {
            duel.damage(1 - tp, tc.get_level(duel) * 100, REASON_EFFECT)?;
        }
    }
    Ok(())
}
