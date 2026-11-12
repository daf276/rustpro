// Bazoo the Soul-Devourer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 79480466;

// Copies in the Extra Deck, Xyz Summoned with three Level 4 monsters (Hyosube, Water Omotics, Uraby), and Raigeki in the hand to
// send the opponent's monsters to the Graveyard, which detaches a material until the last one triggers the attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (2483611, MZONE),
    (1784619, MZONE),
    (1784619, MZONE),
    (12580477, HAND),
    (2118022, MZONE | OPPONENT),
    (2483611, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    duel.enable_global_flag(GLOBALFLAG_DETACH_EVENT);
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_OVERLAY) && !c.is_reason(duel, REASON_LOST_TARGET)
    });
    e1.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default().is_exists(duel, CardId::is_monster_card, 1, None)
    });
    e2.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && a.e.get_handler(duel).check_remove_overlay_card(duel, a.tp, 1, REASON_EFFECT))
    });
    e2.set_operation(duel, detachop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_DETACH_MATERIAL);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    e3.set_target(duel, attachtg);
    e3.set_operation(duel, attachop);
    c.register_effect(duel, e3);
}

fn detachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        c.remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT));
    }
    let filter = |tc: CardId, duel: &Duel| tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 3, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, g.map_or(0, |g| g.len()) as u8, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let mut tg = duel.get_target_cards(e).unwrap_or_default().filter(
        duel,
        |tc, duel| tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
        None,
    );
    tg.remove_matching(duel, |tc, duel| tc.is_immune_to_effect(duel, e), None);
    if !tg.is_empty() {
        duel.overlay(c, tg)?;
    }
    Ok(())
}
