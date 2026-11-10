// Dark Magician of Chaos
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 40737112;

// In the hand to be Tribute Summoned with Prevent Rats, Spells in the Graveyard to add at the end of the turn, and the opponent's
// Elfnote Lucina to destroy in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (12580477, GRAVE),
    (53129443, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_condition_mut(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_faceup(duel));
    e3.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SUMMON_SUCCESS);
        ge1.set_label(duel, i64::from(ID));
        ge1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        ge1.set_operation(duel, aux::sumreg);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        ge2.set_label(duel, i64::from(ID));
        duel.register_effect(ge2, 0);
    });
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) != 0);
    }
    c.reset_flag_effect(duel, ID);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    a.e.set_label_object_group(duel, bc.map(Into::into));
    bc.is_some_and(|bc| bc.is_status(duel, STATUS_BATTLE_DESTROYED)) && c.is_status(duel, STATUS_OPPO_BATTLE)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_REMOVE, bc, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && bc.is_relate_to_battle(duel)
        && bc.is_able_to_remove(duel)
    {
        duel.remove(bc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
