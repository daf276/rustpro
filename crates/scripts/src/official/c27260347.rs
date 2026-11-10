// Snake-Eyes Diabellstar
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 27260347;
const KITCHEN_DRAGONMAID: u32 = 16960120;
const DRAGONMAID_CEHRMBA: u32 = 12163590;

pub const LISTED_NAMES: &[u32] = &[ID];

// Two copies on the field to attack into the opponent's monsters, with a FIRE monster (Kitchen Dragonmaid) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (KITCHEN_DRAGONMAID, GRAVE),
    (KITCHEN_DRAGONMAID, MZONE | OPPONENT),
    (DRAGONMAID_CEHRMBA, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, btplcon);
    e1.set_operation(duel, btplop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_continuous_spell(duel));
    e2.set_target(duel, gypltg);
    e2.set_operation(duel, gyplop);
    c.register_effect(duel, e2);
}

fn checkzones(duel: &Duel, c0: CardId, c1: CardId) -> bool {
    let (p0, p1) = (c0.get_owner(duel), c1.get_owner(duel));
    if p0 == p1 {
        return duel.get_location_count(p0, LOCATION_SZONE) > 1;
    }
    duel.get_location_count(p0, LOCATION_SZONE) > 0 && duel.get_location_count(p1, LOCATION_SZONE) > 0
}

fn btplcon(duel: &Duel, a: &Args) -> bool {
    let (Some(bc0), Some(bc1)) = duel.get_battle_monster(a.tp) else {
        return false;
    };
    bc0 == a.e.get_handler(duel) && checkzones(duel, bc0, bc1)
}

pub fn stplace(duel: &mut Duel, c: CardId, tp: u8, rc: CardId) -> Script<bool> {
    if !duel.move_to_field(c, tp, c.get_owner(duel), LOCATION_SZONE, POS_FACEUP, c.is_monster_card(duel))? {
        return Ok(false);
    }
    let e1 = duel.create_effect(rc);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
    c.register_effect(duel, e1);
    Ok(true)
}

fn btplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let (Some(bc0), Some(bc1)) = duel.get_battle_monster(tp) else {
        return Ok(());
    };
    if bc0.is_relate_to_battle(duel)
        && !bc0.is_immune_to_effect(duel, e)
        && bc1.is_relate_to_battle(duel)
        && !bc1.is_immune_to_effect(duel, e)
        && checkzones(duel, bc0, bc1)
        && stplace(duel, bc0, tp, bc0)?
    {
        stplace(duel, bc1, tp, bc0)?;
    }
    Ok(())
}

fn gyplfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && !c.is_code(duel, ID) && !c.is_forbidden(duel)
}

fn gypltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && gyplfilter(chkc, duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(gyplfilter, tp, LOCATION_GRAVE, 0, 1, None)
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, gyplfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn gyplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let c = e.get_handler(duel);
    if tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) && stplace(duel, tc, tp, c)? && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
