// Speedroid Red-Eyed Dice
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 16725505;

pub const LISTED_NAMES: &[u32] = &[ID];

// Speedroid Terrortop on the field to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (81275020, MZONE), (81275020, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SPEEDROID]) && !c.is_code(duel, ID) && c.has_level(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_LVRANK);
    let level = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_level(duel));
    let lv = duel.announce_number(tp, &(1..=6).filter(|&n| n != u64::from(level)).collect::<Vec<_>>())?;
    duel.set_target_param(lv as u32);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    let lv: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if let Some(tc) = tc
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
