// Metalfoes Silverd
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 7868571;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
// Its effect needs a Metalfoes Spell/Trap in the Deck, and none is ported.
pub const CHECKS: &[Check] = &[Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SET);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if c.is_facedown(duel) {
        return false;
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let ignore = ft == 0 && c.is_location(duel, LOCATION_SZONE) && c.get_sequence(duel) < 5;
    duel.is_existing_matching_card(|c, duel| filter(c, duel, ignore), tp, LOCATION_DECK, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel, ignore: bool) -> bool {
    c.is_set_card(duel, &[SET_METALFOES]) && c.is_spell_trap(duel) && c.is_ssetable_full(duel, ignore, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && desfilter(chkc, duel, tp) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && duel.destroy(tc, REASON_EFFECT)? != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, false), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.sset(tp, sc)?;
        }
    }
    Ok(())
}
