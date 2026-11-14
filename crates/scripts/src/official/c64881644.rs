// Refrain the Melodious Songstress
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, PZONE};

const ID: u32 = 64881644;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS];
pub const LISTED_NAMES: &[u32] = &[ID];

const BACHA: u32 = 56208713;

// In the Pendulum Zone and the hand, with Bacha on the field and in the Extra Deck to Fusion Summon from Aria and Elegy,
// which are also in the Deck to send or add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (BACHA, MZONE),
    (BACHA, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (40502912, MZONE),
    (79514956, MZONE),
    (40502912, DECK),
    (79514956, DECK),
    (79514956, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgatktg);
    e1.set_operation(duel, tgatkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, |duel, a, chk, _| {
        stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
    });
    e2.set_operation(duel, |duel, a| stock::search(duel, a, thfilter, LOCATION_DECK));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    add_pendulum_placement(duel, c, ID);
}

// The Extra Deck effect Refrain and Couplet share: placed in the Pendulum Zone when a Melodious Fusion Monster is summoned.
pub fn add_pendulum_placement(duel: &mut Duel, c: CardId, id: u32) {
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(id, 2));
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_EXTRA);
    e4.set_count_limit(duel, 1, (id, 2));
    e4.set_condition(duel, pencon);
    e4.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e4.set_operation(duel, penop);
    c.register_effect(duel, e4);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_type(duel, TYPE_FUSION) && c.is_faceup(duel)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgatktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(sc) = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.send_to_grave(sc, REASON_EFFECT)? == 0 || !sc.is_location(duel, LOCATION_GRAVE) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, sc.get_level(duel) as i32 * 200);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_monster(duel) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_type(duel, TYPE_FUSION) && c.is_controler(duel, tp)
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
