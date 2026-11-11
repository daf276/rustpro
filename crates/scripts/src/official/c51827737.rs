// Genex Recycled
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 51827737;

// On the field, with a Genex Controller in the Graveyard to copy the name of.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (68505803, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_GENEX]);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, tc.get_code(duel) as i32);
    c.register_effect(duel, e1);
    Ok(())
}
