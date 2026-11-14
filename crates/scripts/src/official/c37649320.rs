// Paleozoic Opabinia
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 37649320;

pub const LISTED_SERIES: &[u16] = &[SET_PALEOZOIC];
// In the Extra Deck, Xyz Summoned with two Level 2 monsters (Paleozoic Marrella revived from the Graveyard by a Trap
// activation becomes one, Trap Monsters being the Trap Xyz Material the search needs), and Paleozoic Traps in the Deck to search
// and in the hand to activate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (64765016, GRAVE),
    (64765016, GRAVE),
    (64765016, SZONE),
    (64765016, HAND),
    (38761908, DECK),
    (38761908, DECK),
    (98414735, DECK),
    (7892180, MZONE),
    (7892180, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_HAND, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_PALEOZOIC]));
    c.register_effect(duel, e3);
}

pub fn efilter(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    re.is_monster_effect(duel) && re.get_owner(duel) != e.get_owner(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, CardId::is_trap, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PALEOZOIC]) && c.is_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
