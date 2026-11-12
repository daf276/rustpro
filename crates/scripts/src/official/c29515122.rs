// Digvorzhak, King of Heavy Industry
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 29515122;

// In the Extra Deck, Xyz Summoned with three Level 5 Normal Monsters, with the opponent's Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2971090, MZONE),
    (4035199, MZONE),
    (4179849, MZONE),
    (4179849, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(1 - tp, 3));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, 1 - tp, 3);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_GRAVE) && c.is_monster(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.discard_deck(1 - tp, 3, REASON_EFFECT)?;
    let g = duel.get_operated_group();
    let ct = g.filter_count(duel, cfilter, None);
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    duel.hint_selection(dg);
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
