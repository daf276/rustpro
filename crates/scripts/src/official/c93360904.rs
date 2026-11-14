// Yummyusment★Acroquey
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 93360904;

pub const LISTED_SERIES: &[u16] = &[SET_YUMMY];
// Level 1 Yummy monsters and Cooky★Yummy Way (a Level 2 LIGHT Beast Synchro) to Synchro Summon, more in the Deck and Graveyard,
// and the opponent's Raigeki to destroy them by effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (67098897, EXTRA),
    (67098897, EXTRA),
    (31425736, MZONE),
    (68810435, MZONE),
    (4215180, DECK),
    (31425736, DECK),
    (68810435, GRAVE),
    (10966439, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    super::c66975205::add_to_deck_effect::<ID, 2>(duel, c);
}

fn desconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_synchro_summoned(duel)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_BEAST)
        && c.is_summon_player(duel, tp)
        && c.is_faceup(duel)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| desconfilter(c, duel, a.tp), 1, None))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::destroy_first_target(duel, a)
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8, rp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && rp == 1 - tp
        && c.is_reason(duel, REASON_EFFECT)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp, a.rp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK, POS_FACEUP)
}
