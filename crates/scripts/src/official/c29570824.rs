// Ekto Enneacraft - "tromarIA"
use super::c28454232::{flip_quick, register_counter, register_set_summon};
use super::c92171126::{register_reveal, thop, thtg};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 29570824;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone, face-down on the field and in the Deck to reveal and search, with the opponent's Monster
// Reborn and Glow-Up Bulb to answer and Sangan to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (83764718, HAND | OPPONENT),
    (67441435, GRAVE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_reveal::<ID>(duel, c);
    register_set_summon::<ID, 1>(duel, c);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_TOHAND + CATEGORY_SEARCH, 0);
    e4.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_facedown(duel)
            && a.ep == 1 - a.tp
            && a.re.is_some_and(|re| re.is_has_category(duel, CATEGORY_SPECIAL_SUMMON))
    });
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_DISABLE);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_FLIP);
    e5.set_operation(duel, disop);
    c.register_effect(duel, e5);
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_matching_card(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(sc) = g.first() {
        duel.hint_selection(sc);
        let c = a.e.get_handler(duel);
        sc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
    }
    Ok(())
}
