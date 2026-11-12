// Fairy Knight Ingunar
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 19684740;

// In the Extra Deck, Xyz Summoned with three Level 6 Normal Monsters (Boulder Tortoise).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (9540040, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, 0, 0);
    duel.set_chain_limit(|_, _, ep, tp, _| tp == ep, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_able_to_hand, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
