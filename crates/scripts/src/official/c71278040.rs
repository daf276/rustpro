// Parallel eXceed
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 71278040;

pub const LISTED_NAMES: &[u32] = &[ID];
// In hand while Link Spider is Link Summoned with Zure, Knight of Dark World, and in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (98978921, EXTRA),
    (98978921, EXTRA),
    (7459013, MZONE),
    (7459013, MZONE),
    (7459013, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_link_summoned(duel) && c.is_summon_player(duel, a.tp), 1, None)
    });
    e1.set_target(duel, handsptg);
    e1.set_operation(duel, handspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_condition(duel, |duel, a| a.re.is_some_and(|re| re.get_handler(duel).is_code(duel, ID)));
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn linked_zones(duel: &Duel, a: &Args) -> u32 {
    let links =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| c.is_summon_type(duel, &[SUMMON_TYPE_LINK]), None);
    links.get_bitwise_or(duel, |c, duel| u64::from(c.get_linked_zone_full(duel, a.tp))) as u32 & ZONES_MMZ
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zones = linked_zones(duel, a);
    if chk == 0 {
        return Ok(zones != 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zones));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let zones = linked_zones(duel, a);
    if c.is_relate_to_effect(duel, a.e) && zones != 0 {
        duel.special_summon_full(c, 0, a.tp, a.tp, false, false, POS_FACEUP, zones)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ID) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_LEVEL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 4);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_SET_BASE_ATTACK);
    let half_attack = (c.get_base_attack(duel) + 1) / 2;
    e2.set_value(duel, half_attack);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_SET_BASE_DEFENSE);
    let half_defense = (c.get_base_defense(duel) + 1) / 2;
    e3.set_value(duel, half_defense);
    c.register_effect(duel, e3);
    Ok(())
}
