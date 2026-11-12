// Adreus, Keeper of Armageddon
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 94119480;

// In the Extra Deck, Xyz Summoned with Level 5 Normal Monsters, with a monster on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4035199, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<5, 2>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
