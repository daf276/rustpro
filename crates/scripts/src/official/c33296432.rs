// Dogmatika Adin, the Enlightened
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 33296432;

pub const LISTED_SERIES: &[u16] = &[SET_DOGMATIKA];
pub const LISTED_NAMES: &[u32] = &[ID];
// On the field to be destroyed by the opponent's Raigeki, with Dogmatika Ecclesia and Lawbringer in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (60303688, DECK),
    (30394645, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon1);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| i32::from(c.is_summon_location(duel, LOCATION_EXTRA))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon2);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA)
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DOGMATIKA]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    a.r & (REASON_EFFECT + REASON_BATTLE) != 0 && a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
