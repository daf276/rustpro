// Elemental HERO Blazeman
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 63060238;
pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

// Two in the hand, with Polymerization and Elemental HERO Sparkman in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_POLYMERIZATION, DECK),
    (CARD_POLYMERIZATION, DECK),
    (20721928, DECK),
    (20721928, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
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
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ELEMENTAL_HERO]) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_value(duel, tc.get_attribute(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e2.set_value(duel, tc.get_attack(duel));
        c.register_effect(duel, e2);
        let e3 = e1.clone_effect(duel);
        e3.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        e3.set_value(duel, tc.get_defense(duel));
        c.register_effect(duel, e3);
    }
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e4.set_target_range(duel, 1, 0);
    e4.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        if c.is_monster(duel) {
            !c.is_type(duel, TYPE_FUSION)
        } else {
            !c.is_original_type(duel, TYPE_FUSION)
        }
    });
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e4, tp);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e5.set_code(duel, CARD_EHERO_BLAZEMAN);
    e5.set_target_range(duel, 1, 0);
    e5.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e5, tp);
    Ok(())
}
