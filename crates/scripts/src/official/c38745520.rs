// Dracoback, the Rideable Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 38745520;

pub const LISTED_NAMES: &[u32] = &[TOKEN_ADVENTURER];

// In the hand, with Dark Magicians (Normal Monsters) to equip to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (46986414, MZONE), (46986414, MZONE), (46986414, MZONE)];
// Only Fateful Adventure makes an Adventurer Token to equip to, and it is not ported yet.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl EquipProcedure for Procedure {
    const P: Option<u8> = Some(0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).check_unique_on_field(duel, a.tp));
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .is_some_and(|tc| tc.is_non_effect_monster(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, TOKEN_ADVENTURER)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_relate_to_effect(duel, e)
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(eqfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_code(duel, TOKEN_ADVENTURER)
        && tc.is_controler(duel, tp)
        && c.check_unique_on_field(duel, tp)
    {
        duel.equip(tp, c, tc)?;
    }
    Ok(())
}
