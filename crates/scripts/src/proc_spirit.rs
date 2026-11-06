// Port of proc_spirit.lua: Spirit.AddProcedure, a Spirit monster's return to the hand in the End Phase after the events
// that set its flag.
use engine::{Args, CardId, Duel, Script};

use crate::constant::*;
use crate::proc_workaround::CardWorkaround;
use crate::utility::CardUtility;

pub struct Spirit;

pub const FLAG_SPIRIT_RETURN: u32 = 2;

impl Spirit {
    pub fn add_procedure(duel: &mut Duel, c: CardId, events: &[u32]) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 1105);
        e1.set_category(duel, CATEGORY_TOHAND);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_condition(duel, Spirit::mandatory_return_condition);
        e1.set_target(duel, Spirit::mandatory_return_target);
        e1.set_operation(duel, Spirit::return_operation);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
        e2.set_condition(duel, Spirit::optional_return_condition);
        e2.set_target(duel, Spirit::optional_return_target);
        c.register_effect(duel, e2);
        for &event in events {
            let fe1 = duel.create_effect(c);
            fe1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            fe1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            fe1.set_code(duel, event);
            fe1.set_promptless_operation(duel, |duel, a| {
                a.e.get_handler(duel)
                    .register_flag_effect(duel, FLAG_SPIRIT_RETURN, RESETS_STANDARD_PHASE_END, 0, 1);
                Ok(())
            });
            c.register_effect(duel, fe1);
        }
    }

    pub fn common_condition(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        c.has_flag_effect(duel, FLAG_SPIRIT_RETURN) && c.is_has_effect(duel, EFFECT_SPIRIT_DONOT_RETURN).is_none()
    }

    pub fn mandatory_return_condition(duel: &Duel, a: &Args) -> bool {
        Spirit::common_condition(duel, a) && a.e.get_handler(duel).is_has_effect(duel, EFFECT_SPIRIT_MAYNOT_RETURN).is_none()
    }

    pub fn mandatory_return_target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let c = a.e.get_handler(duel);
        c.reset_flag_effect(duel, FLAG_SPIRIT_RETURN);
        duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
        Ok(false)
    }

    pub fn optional_return_condition(duel: &Duel, a: &Args) -> bool {
        Spirit::common_condition(duel, a) && a.e.get_handler(duel).is_has_effect(duel, EFFECT_SPIRIT_MAYNOT_RETURN).is_some()
    }

    pub fn optional_return_target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        let c = a.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_hand(duel));
        }
        c.reset_flag_effect(duel, FLAG_SPIRIT_RETURN);
        duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
        Ok(false)
    }

    pub fn return_operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) {
            duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        }
        Ok(())
    }
}
