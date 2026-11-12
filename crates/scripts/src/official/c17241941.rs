// Gigathunder Giclops
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 17241941;

// Two copies in the Pendulum Zones (one is destroyed by Mystical Space Typhoon), one on the field and two in the hand to Normal Summon,
// with a monster and a Trap of the opponent's to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, PZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (549481, MZONE | OPPONENT),
    (9765723, SZONE | OPPONENT),
    (5318639, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_category(duel, CATEGORY_POSITION);
    e0.set_type(duel, EFFECT_TYPE_IGNITION);
    e0.set_range(duel, LOCATION_PZONE);
    e0.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e0.set_count_limit(duel, 1, 0);
    e0.set_target(duel, pentg);
    e0.set_operation(duel, penop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    e1.set_condition(duel, rmcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, 0);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_CHANGE_POS);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, sendcon);
    e4.set_target(duel, sendtg);
    e4.set_operation(duel, sendop);
    c.register_effect(duel, e4);
}

fn penfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_change_position(duel) && c.is_type(duel, TYPE_PENDULUM)
}

fn oppofilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_change_position(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(penfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(oppofilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let mut g1 = duel
        .select_target(tp, penfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g2 = duel
        .select_target(tp, oppofilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g1.merge(g2);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g1), 2, 0, 0);
    Ok(false)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.change_position_full(
            g,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_summon_location(duel, LOCATION_EXTRA)
        && c.get_reason(duel) & (REASON_MATERIAL + REASON_SYNCHRO) == REASON_MATERIAL + REASON_SYNCHRO
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.change_position_full(
            c,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn sendcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_PZONE)
}

fn sendtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_PZONE, 0) > 0);
    }
    let g = duel.get_field_group(tp, LOCATION_PZONE, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), 1, 0, 0);
    Ok(false)
}

fn sendop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.get_field_group(tp, LOCATION_PZONE, 0).select(duel, tp, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(g);
    duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
