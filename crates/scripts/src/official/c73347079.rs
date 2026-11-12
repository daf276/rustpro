// Raidraptor - Force Strix
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 73347079;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// On the field, with Blackwing - Zephyros the Elite and Tri-Brigade Mercourier (Level 4 DARK Winged Beasts) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (14785765, DECK), (14785765, DECK), (19096726, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(adval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn adval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let winged = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_WINGEDBEAST));
    duel.get_matching_group_count(winged, c.get_controler(duel), LOCATION_MZONE, 0, Some(c.into())) as i32 * 500
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.get_level(duel) == 4 && c.is_race(duel, RACE_WINGEDBEAST) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
