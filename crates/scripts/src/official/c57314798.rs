// Number 100: Numeron Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 57314798;
pub const XYZ_NUMBER: u32 = 100;

// In the Extra Deck, Xyz Summoned with two Number 39: Utopia, with the opponent's Dark Hole to destroy it and a Raigeki in each Graveyard to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (84013237, MZONE),
    (84013237, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, GRAVE),
    (12580477, GRAVE | OPPONENT),
];

// Needs an empty hand and field while the opponent attacks directly, which random play never reaches.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(xyzfilter);
    const LV: Option<u32> = None;
    const CT: usize = 2;

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        let mg = g.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EQUIP_SPELL_XYZ_MAT).is_none(), None);
        mg.get_class_count(duel, |c, duel| i64::from(c.get_rank(duel))) == 1
            && mg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, stock::special_summon_self_target_player_0);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn xyzfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(xyz), sumtype, tp) && c.is_set_card_full(duel, &[SET_NUMBER], Some(xyz), sumtype, tp)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.get_rank(duel) > 0
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_faceup(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let atk = g.get_sum(duel, |c, duel| i64::from(c.get_rank(duel)));
    if atk <= 0 {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, atk as i32 * 1000);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END | RESET_OPPO_TURN);
    c.register_effect(duel, e1);
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_spell_trap(duel) && c.is_ssetable_full(duel, false, Some(tp))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let count = g.len() as u8;
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), count, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 2, PLAYER_ALL, LOCATION_GRAVE as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let tc1 = duel
        .select_matching_card(tp, |c, duel| setfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first();
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SET);
    let tc2 = duel
        .select_matching_card(1 - tp, |c, duel| setfilter(c, duel, 1 - tp), 1 - tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first();
    if tc1.is_some_and(|tc| tc.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_some())
        || tc2.is_some_and(|tc| tc.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_some())
    {
        return Ok(());
    }
    if let Some(tc1) = tc1 {
        duel.sset(tp, tc1)?;
    }
    if let Some(tc2) = tc2 {
        duel.sset(1 - tp, tc2)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_status(duel, STATUS_LEAVE_CONFIRMED)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - tp))
        && duel.get_attack_target().is_none()
        && !duel.is_existing_matching_card(spfilter, tp, LOCATION_ONFIELD | LOCATION_HAND, 0, 1, None)
}
