// Old Entity Hastorr
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 70913714;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Laval Coatl (Level 2 Tuner) with Fire Eye for a Synchro Summon from the Extra Deck, destroyed by the opponent's Raigeki with
// Normal Monsters left on its side to equip to and take control of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (45439263, MZONE),
    (88435542, MZONE),
    (45042329, MZONE | OPPONENT),
    (45042329, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (5318639, HAND),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, eqcon);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn eqlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(e.get_owner(duel) == c)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        duel.equip_full(tp, c, tc, true, false)?;
        let e1 = duel.create_effect(tc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(eqlimit));
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_DISABLE);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
        let e3 = e2.clone_effect(duel);
        e3.set_code(duel, EFFECT_CANNOT_ATTACK);
        c.register_effect(duel, e3);
        let e4 = duel.create_effect(c);
        e4.set_description(duel, aux::stringid(ID, 1));
        e4.set_category(duel, CATEGORY_CONTROL);
        e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
        e4.set_code(duel, EVENT_LEAVE_FIELD);
        e4.set_condition(duel, ctcon);
        e4.set_target(duel, cttg);
        e4.set_operation(duel, ctop);
        e4.set_reset(duel, RESET_EVENT | RESET_TURN_SET | RESET_TOFIELD);
        c.register_effect(duel, e4);
    }
    Ok(())
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_reason(duel, REASON_LOST_TARGET)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ec = a.e.get_handler(duel).get_previous_equip_target(duel);
    unported!(duel, ec.is_none());
    if let Some(ec) = ec
        && ec.is_location(duel, LOCATION_MZONE)
        && ec.is_controler_can_be_changed(duel)
    {
        duel.set_target_card(ec);
        duel.set_operation_info(0, CATEGORY_CONTROL, Some(ec.into()), 1, 0, 0);
    }
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(ec) = duel.get_first_target()
        && ec.is_relate_to_effect(duel, a.e)
    {
        duel.get_control(ec, a.tp)?;
    }
    Ok(())
}
