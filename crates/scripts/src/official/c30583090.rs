// Evil HERO Dead-End Prison
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 30583090;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_FUSION];

// In the hand and on the field, with Evil HERO Dark Gaia on the field to Special Summon it, a HERO in the Deck for the cost,
// and Elemental HERO Flame Wingman in the Extra Deck to be Fusion Summoned by Polymerization with Avian and the Prison.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (58332301, MZONE),
    (21844576, DECK),
    (21844576, HAND),
    (CARD_POLYMERIZATION, HAND),
    (35809262, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, fsubcost);
    e2.set_operation(duel, fsubop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    super::dark_calling(c.get_original_code(duel)) && c.is_type(duel, TYPE_FUSION) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn fsubcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERO]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn fsubcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, fsubcostfilter, LOCATION_DECK)
}

fn fsubop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_FUSION_SUBSTITUTE);
        e1.set_value(duel, Value::Card(|duel, _, fc| i32::from(fc.is_set_card(duel, &[SET_HERO]))));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
