// Genex Ally Chemistrer
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 38049541;

// In the hand, with Genex Controller (a Level 3 Normal Monster) and R-Genex Overseer on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (68505803, MZONE), (68505803, MZONE), (32744558, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, attrtg);
    e1.set_operation(duel, attrop);
    c.register_effect(duel, e1);
}

fn genex(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GENEX])
}

fn attrtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, tp)
            && genex(chkc, duel)
            && chkc.is_attribute_except(duel, e.get_label(duel) as u32));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(genex, tp, LOCATION_MZONE, 0, 1, None));
    }
    let g = duel.get_matching_group(
        |c, duel| c.is_can_be_effect_target(duel, e) && genex(c, duel),
        tp,
        LOCATION_MZONE,
        0,
        None,
    );
    let att = duel.announce_another_attribute(g, tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let sel = g.filter_select(duel, tp, |c, duel| c.is_attribute_except(duel, att), 1, 1, None)?;
    duel.set_target_card(sel);
    e.set_label(duel, att.into());
    Ok(false)
}

fn attrop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e1.set_value(duel, e.get_label(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
