// Anarchist Monk of the Six Samurai
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 80570228;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand beside Legendary Six Samurai - Kageki to Special Summon, with the opponent's Raigeki to send it to the Graveyard
// and Asceticism of the Six Samurai (a Quick-Play Spell) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (2511717, MZONE),
    (27821104, DECK),
    (27821104, DECK),
    (27821104, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    // The hint goes on the Synchro Monster, which a CardHint cannot count for this card; no ported Six Samurai Synchro Monster takes
    // this Level 3 Tuner (Level 2 non-Tuners are missing).
    Check::Report(&Check::CardHint(aux::stringid(ID, 2))),
];

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
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| {
        a.r & REASON_SYNCHRO == REASON_SYNCHRO
            && a.e
                .get_handler(duel)
                .get_reason_card(duel)
                .is_some_and(|rc| rc.is_set_card(duel, &[SET_SIX_SAMURAI]))
    });
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.is_faceup(duel) && !c.is_code(duel, ID)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.is_quick_play_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(rc);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, -1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    if !rc.is_type(duel, TYPE_EFFECT) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_ADD_TYPE);
        e2.set_value(duel, TYPE_EFFECT as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect_full(duel, e2, true);
    }
    rc.register_flag_effect_full(
        duel,
        ID,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 2),
    );
    Ok(())
}
