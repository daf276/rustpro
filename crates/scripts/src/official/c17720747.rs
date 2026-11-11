// Witch of the Black Rose
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 17720747;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_ONFIELD, 0) <= 1
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tc = duel.get_decktop_group(tp, 1).get_first();
    duel.draw(tp, 1, REASON_EFFECT)?;
    if let Some(tc) = tc {
        duel.confirm_cards(1 - tp, tc)?;
        if !tc.is_monster(duel) {
            duel.break_effect()?;
            duel.send_to_grave(tc, REASON_EFFECT)?;
            let c = a.e.get_handler(duel);
            if c.is_relate_to_effect(duel, a.e) {
                duel.destroy(c, REASON_EFFECT)?;
            }
        }
        duel.shuffle_hand(tp)?;
    }
    Ok(())
}
