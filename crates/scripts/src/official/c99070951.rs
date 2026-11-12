// Esper Girl
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, REMOVED};

const ID: u32 = 99070951;

// Copies banished for D.D.R. - Different Dimension Reincarnation to Special Summon, and the opponent's Raigeki to send them to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, REMOVED),
    (ID, REMOVED),
    (9622164, HAND),
    (9622164, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, rmcon);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_REMOVE, LOCATION_DECK>);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_REMOVED)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_decktop_group(tp, 1);
    let Some(tc) = g.get_first() else { return Ok(()) };
    if !tc.is_able_to_remove(duel) {
        return Ok(());
    }
    duel.disable_shuffle_check(true);
    duel.remove(tc, POS_FACEDOWN, REASON_EFFECT)?;
    tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    if c.is_location(duel, LOCATION_MZONE) {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESET_TODECK | RESET_TOHAND | RESET_REMOVE, 0, 1);
    }
    e.set_label_object_group(duel, Some(tc.into()));
    Ok(())
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object(duel)?.get_label_object_group(duel)?.get_first()
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.get_flag_effect(duel, ID) != 0
        && label_card(duel, a.e).is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tc = label_card(duel, a.e);
    duel.set_operation_info(0, CATEGORY_TOHAND, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = label_card(duel, a.e)
        && tc.get_flag_effect(duel, ID) != 0
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
