// Number 11: Big Eye
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 80117527;
pub const XYZ_NUMBER: u32 = 11;

// In the Extra Deck, Xyz Summoned with Gaia The Fierce Knight, Seiyaryu and Ryu-Ran, and an opponent's monster to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (6368038, MZONE),
    (6740720, MZONE),
    (2964201, MZONE),
    (15025844, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, ctrlcost])
    });
    e1.set_target(duel, ctrltg);
    e1.set_operation(duel, ctrlop);
    c.register_effect(duel, e1);
}

fn ctrlcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_attack_announced_count(duel) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(false)
}

fn ctrltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn ctrlop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.get_control(tc, a.tp)?;
    }
    Ok(())
}
