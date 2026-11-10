// Hiita the Fire Charmer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 759393;

// Face-down on the field, flipped by Elfnote Lucina's attack, with Charcoal Inpachi and Volcanic Rat to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13179332, MZONE | OPPONENT),
    (33112041, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ATTRIBUTE_FIRE>(duel, c, ID);
}

// The flip effect the six elemental Charmers share, differing only in the Attribute.
pub fn register<const ATTRIBUTE: u32>(duel: &mut Duel, c: CardId, id: u32) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(id, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, target::<ATTRIBUTE>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE) && c.is_controler_can_be_changed(duel)
}

fn target<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter::<ATTRIBUTE>(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    if !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let g = duel.select_target(tp, filter::<ATTRIBUTE>, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_CONTROL, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        c.set_card_target(duel, tc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_CONTROL);
        e1.set_value(duel, i32::from(tp));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_condition(duel, ctcon);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_owner(duel).is_has_card_target(duel, a.e.get_handler(duel))
}
