// Labrynth Labyrinth
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 33407125;

pub const LISTED_SERIES: &[u16] = &[SET_LABRYNTH, SET_WELCOME_LABRYNTH];

// In the hand to activate, with Waboku and Raigeki Break to activate, Fiends in the hand and Graveyard to summon, and Welcome
// Labrynth with Arias in the Deck to summon, for the destruction it adds.
pub const SUPPORT: &[(u32, u16)] = &[
    (73602965, DECK),
    (73602965, DECK),
    (ID, 0),
    (ID, HAND),
    (12607053, SZONE),
    (12607053, HAND),
    (12607053, HAND),
    (4178474, HAND),
    (5380979, HAND),
    (5380979, HAND),
    (5434080, GRAVE),
    (7459013, HAND),
    (73602965, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, CARD_LABRYNTH_LABYRINTH);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target_range(duel, 1, 0);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let c = re.get_handler(duel);
    a.rp == a.tp
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_active_type(duel) == TYPE_TRAP
        && !c.is_set_card(duel, &[SET_LABRYNTH])
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
