// Ib the World Chalice Priestess
use super::c31226177::{spcon2, spop2, sptg2};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 77610772;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_CHALICE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_race(duel))
            && aux::check_different_property_binary(duel, sg, &|c, duel| u64::from(c.get_attribute(duel)))
    }
}

// In the Extra Deck, Link Summoned with Sangans (DARK Fiend) and Digitrons (EARTH Cyberse), with Lee the World Chalice Fairy in the hand
// to Special Summon, and Raigeki from the opponent's hand to destroy the monsters it points to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (32295838, MZONE),
    (32295838, MZONE),
    (21893603, HAND),
    (21893603, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, reptg);
    e4.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel), e.get_handler(duel)))),
    );
    e4.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.send_to_grave(c, REASON_EFFECT).map(drop)
    });
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_condition(duel, spcon2);
    e5.set_target(duel, sptg2);
    e5.set_operation(duel, spop2);
    c.register_effect(duel, e5);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8, hc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_location(duel, LOCATION_MZONE)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
        && hc.get_linked_group(duel).contains(c)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave(duel)
            && a.eg
                .unwrap_or_default()
                .is_exists(duel, |sc, duel| repfilter(sc, duel, tp, c), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
