// Cyberse Wicckid
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 52698008;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Code Generators, then Backup @Ignisters in the hand to Special Summon to its zones and in the
// Deck to search, and Micro Coders in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (30114823, MZONE),
    (30114823, MZONE),
    (30118811, HAND),
    (30118811, DECK),
    (30118811, DECK),
    (30118811, DECK),
    (2347477, GRAVE),
    (2347477, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, indtg);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_count_limit(duel, 1, ID);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, thcon);
    e4.set_cost(duel, thcost);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_link_summoned(duel)
}

fn indtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_race(duel, RACE_CYBERSE) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_controler(duel, a.tp) && lg.is_contains(c), 1, None))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_able_to_remove_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, cfilter, LOCATION_GRAVE)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_type(duel, TYPE_TUNER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
