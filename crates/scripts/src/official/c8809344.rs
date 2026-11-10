// Outer Entity Nyarla
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 8809344;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Level 4 monsters on the field and cards in the hand to discard, and monsters in the Graveyard to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (9491461, MZONE),
    (9491461, MZONE),
    (23720856, MZONE),
    (23720856, MZONE),
    (26202165, HAND),
    (26202165, HAND),
    (26202165, GRAVE),
    (26202165, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_cost(duel, rkcost);
    e1.set_operation(duel, rkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_type(duel, TYPE_XYZ) && c.get_overlay_count(duel) > 0
    });
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn rkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    let ct = duel.discard_hand(tp, CardId::is_discardable, 1, 60, REASON_COST | REASON_DISCARD)?;
    a.e.set_label(duel, i64::from(ct));
    Ok(false)
}

fn rkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let label = a.e.get_label(duel);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_UPDATE_RANK);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e2.set_value(duel, label as i32);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc.is_monster(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_monster, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel
        .select_target(tp, CardId::is_monster, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(g), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e) = (a.e.get_handler(duel), a.e);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let og = c.get_overlay_group(duel);
    if og.is_empty() {
        return Ok(());
    }
    duel.send_to_grave(og, REASON_EFFECT)?;
    duel.overlay(c, tc)?;
    if c.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_value(duel, tc.get_original_attribute(duel) as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CHANGE_RACE);
    e2.set_value(duel, tc.get_original_race(duel) as i32);
    c.register_effect(duel, e2);
    Ok(())
}
