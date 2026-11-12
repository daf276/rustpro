// Number 70: Malevolent Sin
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 80796456;
pub const XYZ_NUMBER: u32 = 70;

// Copies on the field to attack, with weak Sangans of the opponent's to banish and battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| duel.get_attacker() == Some(a.e.get_handler(duel)));
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, 0, REASON_EFFECT | REASON_TEMPORARY)? != 0 {
        tc.register_flag_effect(
            duel,
            ID,
            RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_OPPO_TURN,
            0,
            1,
        );
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, retcon);
        e1.set_operation(duel, retop);
        e1.set_reset(duel, RESET_PHASE | PHASE_STANDBY | RESET_OPPO_TURN);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn retcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && a.e
            .get_label_object_group(duel)
            .and_then(|g| g.get_first())
            .is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(tc)?;
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 300);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_UPDATE_RANK);
        e2.set_value(duel, 3);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e2);
    }
    Ok(())
}
