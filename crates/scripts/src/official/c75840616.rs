// Bujintei Susanowo
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 75840616;

pub const LISTED_SERIES: &[u16] = &[SET_BUJIN];
// In the Extra Deck, Xyz Summoned with Level 4 Bujin Yamato, with more in the Deck to search; it attacks all of the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32339440, MZONE),
    (32339440, MZONE),
    (3606209, MZONE),
    (32339440, DECK),
    (32339440, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_ATTACK)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, _, _, _| c.is_set_card(duel, &[SET_BUJIN]));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DECKDES);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None))
    });
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_ALL);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BUJIN]) && c.is_monster(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.send_to_grave(sc, REASON_EFFECT).map(drop),
            574,
        )?;
    }
    Ok(())
}
