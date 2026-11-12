// Traptrix Pinguicula
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 1688285;

// On the field with Xyz Import to attach Sangan, a Traptrix monster (Atrax) in the Deck to search, and Raigeki to send the opponent's
// monsters to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (55428242, DECK),
    (55428242, DECK),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_REMOVE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, ovtg);
    e3.set_operation(duel, ovop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e4.set_code(duel, EVENT_TO_GRAVE);
    c.register_effect(duel, e4);
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    if te.is_trap_effect(duel) {
        return true;
    }
    let (tc, c) = (te.get_owner(duel), e.get_handler(duel));
    c != tc
        && te.is_activated(duel)
        && te.is_monster_effect(duel)
        && c.get_overlay_group(duel)
            .is_exists(duel, |oc, duel| oc.is_race(duel, tc.get_race(duel)), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRAPTRIX]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn ovfilter(c: CardId, duel: &Duel, e: EffectId, xc: CardId, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_faceup(duel)
        && c.is_reason(duel, REASON_EFFECT)
        && c.get_owner(duel) == 1 - tp
        && c.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
        && !c.is_immune_to_effect(duel, e)
        && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
}

fn ovtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ) && eg.is_exists(duel, |tc, duel| ovfilter(tc, duel, e, c, tp), 1, None));
    }
    duel.set_target_card(eg);
    if e.get_code(duel) == EVENT_TO_GRAVE {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, 1 - tp, 0);
    }
    Ok(false)
}

fn ovop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || !c.is_type(duel, TYPE_XYZ) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let tg = duel
        .get_target_cards(e)
        .unwrap_or_default()
        .filter(duel, |tc, duel| tc.is_relate_to_effect(duel, e), None);
    if tg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    let g = tg.filter_select(duel, tp, |tc, duel| ovfilter(tc, duel, e, c, tp), 1, 1, None)?;
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}
