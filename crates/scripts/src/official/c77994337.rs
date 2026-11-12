// Hallohallo
// s.roll_dice, which only cards that are not ported read, is left out.
use crate::prelude::*;
use crate::testing::{MZONE, PZONE};

const ID: u32 = 77994337;

// In the Pendulum Zone, with monsters on the field to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (48531733, MZONE), (5628232, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && lvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let Some(&dc) = duel.toss_dice(tp, 1)?.first() else {
            return Ok(());
        };
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL_FINAL);
        e1.set_value(duel, i32::from(dc));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
