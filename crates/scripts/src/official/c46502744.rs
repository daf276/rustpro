// Retaliating "C"
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 46502744;

pub const LISTED_NAMES: &[u32] = &[ID];

// A copy in the hand to respond to the opponent's Monster Reborn (with monsters in the Graveyard) and one on the field to
// be destroyed by their Dark Hole, with Petit Moth in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (58192742, DECK),
    (58192742, DECK),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (26976414, GRAVE | OPPONENT),
    (14851496, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e2.set_target_range(duel, 0xff, 0xff);
    e2.set_target_card(duel, |duel, e, c| duel.is_player_can_remove(e.get_handler_player(duel), Some(c)));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID + 1) != 0);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.ep != a.tp
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.is_spell_effect(duel)
        && re.is_has_category(duel, CATEGORY_SPECIAL_SUMMON)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        c.register_flag_effect(duel, ID + 1, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD), 0, 1);
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_INSECT)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.is_attack_below(duel, 1500)
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
