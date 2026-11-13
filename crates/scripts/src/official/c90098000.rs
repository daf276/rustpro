// General of Atlantis
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 90098000;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON];

// Copies in the hand and the Graveyard, with Daedalus Maelstrom (which mentions Atlantis) in the Deck and Levia-Dragon of Atlantis -
// Daedalus on the field, which is named Umi.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (64233143, DECK),
    (64233143, DECK),
    (64603351, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, CARD_UMI as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, Cost::self_discard);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, stock::special_summon_self_target);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, CARD_UMI)),
        a.tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}
