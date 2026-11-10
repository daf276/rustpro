// 7 Completed
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 86198326;

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_MACHINE)
    }

    fn op(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        let tc = duel.get_first_target();
        unported!(duel, tc.is_none());
        if let Some(tc) = tc
            && c.is_relate_to_effect(duel, e)
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
        {
            let opt = duel.select_option(tp, &[aux::stringid(ID, 0), aux::stringid(ID, 1)])?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_EQUIP);
            e1.set_code(duel, if opt == 0 { EFFECT_UPDATE_ATTACK } else { EFFECT_UPDATE_DEFENSE });
            e1.set_value(duel, 700);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e1);
        }
        Ok(())
    }
}

// In the hand, to be activated on a Machine: Unicycular.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (57308711, MZONE), (57308711, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
}
