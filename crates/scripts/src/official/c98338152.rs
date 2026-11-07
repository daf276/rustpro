// Sky Striker Mecha - Widow Anchor
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 98338152;

// In the hand with three Spells in the GY to take control.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (ID, GRAVE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(|c, duel| c.is_in_main_mzone(duel, None), a.tp, LOCATION_MZONE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel)
}

fn three_spells(duel: &Duel, tp: u8) -> bool {
    duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if three_spells(duel, tp) {
        e.set_category(duel, CATEGORY_DISABLE | CATEGORY_CONTROL);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && !tc.is_disabled(duel) {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
        duel.adjust_instantly(Some(tc));
        if three_spells(duel, tp)
            && tc.is_controler_can_be_changed(duel)
            && tc.is_controler(duel, 1 - tp)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.break_effect()?;
            duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
        }
    }
    Ok(())
}
