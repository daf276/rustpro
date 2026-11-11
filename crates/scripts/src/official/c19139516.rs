// Scrap Beast
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 19139516;

pub const LISTED_NAMES: &[u32] = &[ID];
// On the field in Defense Position against Skull Servants too weak to destroy it, with Scrap Raptor to destroy it and Scrap monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (80529459, MZONE),
    (16449363, GRAVE),
    (4334811, GRAVE),
    (4334811, GRAVE),
    (32274490, MZONE | OPPONENT),
    (32274490, MZONE | OPPONENT),
    (32274490, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_operation(duel, regop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, destg::<ID>);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg::<ID>);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

pub(super) fn regop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_defense_pos(duel) && c.is_faceup(duel) {
        c.register_flag_effect(duel, CODE, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE, 0, 1);
    }
    Ok(())
}

pub(super) fn destg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, CODE) > 0);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    Ok(false)
}

pub(super) fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}

pub(super) fn thcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_EFFECT) && c.is_location(duel, LOCATION_GRAVE) && re.get_handler(duel).is_set_card(duel, &[SET_SCRAP])
}

pub(super) fn thfilter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SCRAP]) && c.is_monster(duel) && c.is_able_to_hand(duel) && !c.is_code(duel, CODE)
}

pub(super) fn thtg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter::<CODE>(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter::<CODE>, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter::<CODE>, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

pub(super) fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
