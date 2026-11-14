// Astellar of the White Forest
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 25592142;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];
// On the field and in the Graveyard, with a Spell as its cost and Silvy of the White Forest in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (98385955, DECK), (98385955, DECK), (55144522, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, dspcost);
    e1.set_target(duel, dsptg);
    e1.set_operation(duel, dspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfspcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn dspcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_grave_as_cost(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn dspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| dspcostfilter(c, duel, tp),
            tp,
            LOCATION_HAND | LOCATION_ONFIELD,
            0,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| dspcostfilter(c, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn dspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_SPELLCASTER)
        && c.is_type(duel, TYPE_TUNER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn dsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| dspfilter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn dspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| dspfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn selfspconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_spell_trap(duel) && c.is_reason(duel, REASON_COST) && c.is_controler(duel, tp)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let (eg, tp) = (a.eg.unwrap_or_default(), a.tp);
    let Some(re) = a.re else { return false };
    !eg.is_contains(a.e.get_handler(duel))
        && eg.is_exists(duel, |c, duel| selfspconfilter(c, duel, tp), 1, None)
        && re.is_monster_effect(duel)
        && re.is_activated(duel)
}
