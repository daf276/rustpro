// Firewall Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 5043010;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
}

// Copies on the field to link each other, Link Summoned from the Extra Deck with Leotron and Digitron (Cyberse), and a linked
// monster leaving the field, with Cyberse monsters in the hand and the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (47226949, MZONE),
    (47226949, MZONE),
    (32295838, MZONE),
    (47226949, HAND),
    (32295838, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_NO_TURN_RESET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, regcon2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_DELAY);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_code(duel, EVENT_CUSTOM + ID);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let ct = c.get_mutual_linked_group(duel).filter_count(duel, CardId::is_monster, None);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(ct > 0
            && duel.is_existing_target(
                thfilter,
                tp,
                LOCATION_MZONE | LOCATION_GRAVE,
                LOCATION_MZONE | LOCATION_GRAVE,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(
        tp,
        thfilter,
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        LOCATION_MZONE | LOCATION_GRAVE,
        1,
        ct as u8,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    c.register_flag_effect_full(
        duel,
        0,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 2),
    );
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = duel.get_target_cards(a.e) {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let seq = c.get_previous_sequence(duel) + if c.is_previous_controler(duel, tp) { 0 } else { 16 };
    c.is_previous_location(duel, LOCATION_MZONE) && zone >> seq & 1 != 0
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let zone = a.e.get_handler(duel).get_linked_zone(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp, zone), 1, None))
}

fn regcon2(duel: &Duel, a: &Args) -> bool {
    let zone = a.e.get_handler(duel).get_linked_zone(duel);
    a.eg.is_some_and(|eg| {
        eg.is_exists(
            duel,
            |c, duel| !c.is_reason(duel, REASON_BATTLE) && cfilter(c, duel, a.tp, zone),
            1,
            None,
        )
    })
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, 0, 0)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && c.is_race(duel, RACE_CYBERSE)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}
