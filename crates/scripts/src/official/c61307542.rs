// Paleozoic Anomalocaris
use super::c37649320::efilter;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 61307542;

// In the Extra Deck, Xyz Summoned with three Level 2 monsters (Paleozoic Marrella revived from the Graveyard by a Trap
// activation becomes one, Trap Monsters being the Trap Xyz Material the effects need), and Paleozoic Traps in the Deck to
// excavate and in the hand and Set to activate, and the opponent's Raigeki to destroy them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (64765016, GRAVE),
    (64765016, GRAVE),
    (64765016, SZONE),
    (64765016, HAND),
    (38761908, DECK),
    (38761908, DECK),
    (7892180, MZONE),
    (7892180, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, descon);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, destg);
    e3.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_trap(duel) && c.is_previous_controler(duel, tp) && c.is_previous_location(duel, LOCATION_STZONE)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let tc = duel.get_decktop_group(tp, 1).get_first();
        return Ok(duel.is_player_can_discard_deck(tp, 1) && tc.is_some_and(|tc| tc.is_able_to_hand(duel)));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    duel.confirm_decktop(tp, 1)?;
    let Some(tc) = duel.get_decktop_group(tp, 1).get_first() else {
        return Ok(());
    };
    duel.disable_shuffle_check(true);
    if tc.is_trap(duel) && tc.is_able_to_hand(duel) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.shuffle_hand(tp)?;
    } else {
        duel.send_to_grave(tc, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, CardId::is_trap, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
