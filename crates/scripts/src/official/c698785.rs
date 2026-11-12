// Thunder End Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 698785;

// In the Extra Deck, Xyz Summoned with Level 8 Normal Monsters (Black Luster Soldier, Tri-Horned Dragon, Gogiga Gagagigo), with
// a monster on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (39111158, MZONE),
    (39674352, MZONE),
    (39674352, MZONE),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_type_full(duel, TYPE_NORMAL, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
