// Sky Striker Maneuver - Jamming Waves!
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, SZONE};

const ID: u32 = 25955749;

// In the hand with Set Spells to destroy and three Spells in the GY for the second destruction.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (98338152, GRAVE), (98338152, GRAVE), (98338152, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.get_sequence(duel) < 5
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? != 0 {
        let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        if !dg.is_empty()
            && duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3
            && duel.select_yes_no(tp, aux::stringid(ID, 0))?
        {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = dg.select(duel, tp, 1, 1, None)?;
            duel.hint_selection(sg);
            duel.destroy(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
