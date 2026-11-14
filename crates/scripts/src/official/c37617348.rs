// Rescue-ACE Hydrant
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 37617348;

pub const LISTED_SERIES: &[u16] = &[SET_RESCUE_ACE];
pub const LISTED_NAMES: &[u32] = &[ID];

const RESCUE_ACE_IMPULSE: u32 = 38339996;
const RESCUE_ACE_TURBULENCE: u32 = 37495766;

// On the field and in the hand with Rescue-ACE Impulse, Rescue-ACE monsters in the Deck to search, and Rescue-ACE Turbulence
// on the field to Set EXTINGUISH! and CONTAIN! from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (RESCUE_ACE_IMPULSE, MZONE),
    (RESCUE_ACE_IMPULSE, DECK),
    (RESCUE_ACE_TURBULENCE, DECK),
    (RESCUE_ACE_TURBULENCE, MZONE),
    (99162522, DECK),
    (62777823, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, con);
    e1.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_SZONE, 0);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_target_card(duel, |duel, _, c| c.has_flag_effect(duel, ID));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_QP_ACT_IN_SET_TURN);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_target(duel, thtg);
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
    aux::global_check(duel, ID, |duel| {
        let ge = duel.create_effect(c);
        ge.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge.set_code(duel, EVENT_SSET);
        ge.set_operation(duel, checkop);
        duel.register_effect(ge, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.re.is_some_and(|re| re.get_handler(duel).is_set_card(duel, &[SET_RESCUE_ACE])) {
        return Ok(());
    }
    let Some(eg) = a.eg else { return Ok(()) };
    let sg = eg.filter(duel, |c, duel| c.is_type(duel, TYPE_QUICKPLAY | TYPE_TRAP), None);
    for ec in sg {
        ec.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_RESCUE_ACE]) && !c.is_code(duel, ID)
}

fn con(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RESCUE_ACE]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
