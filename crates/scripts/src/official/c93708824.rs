// Roxrose Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 93708824;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_BLACK_ROSE_DRAGON];
// In the hand and Graveyard, with Rose Dragons (Red Rose, White Rose) on the field for an opponent's Dark Hole to destroy, and
// cards naming Black Rose Dragon (Red Rose Dragon, Blooming of the Darkest Rose) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (26118970, MZONE),
    (12213463, MZONE),
    (53129443, HAND | OPPONENT),
    (26118970, DECK),
    (99092624, DECK),
    (99092624, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_CAL + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, rthcon);
    e3.set_target(duel, rthtg);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && !c.is_code(duel, ID) && c.lists_code(duel, CARD_BLACK_ROSE_DRAGON)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn rthfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_reason(duel, REASON_BATTLE) || (c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp))
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && (c.is_previous_set_card(duel, &[SET_ROSE_DRAGON])
            || (c.get_previous_race_on_field(duel) & RACE_PLANT != 0 && c.get_previous_type_on_field(duel) & TYPE_SYNCHRO != 0))
}

fn rthcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    !eg.contains(c) && eg.is_exists(duel, |tc, duel| rthfilter(tc, duel, a.tp), 1, Some(c.into()))
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}
