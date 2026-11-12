// D/D/D Deviser King Deus Machinex
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 46593546;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_FIEND, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(10);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

// On the field with D/D/D Gust King Alexander to Xyz Summon over, D/D Orthros in a Pendulum Zone to Special Summon, Xyz Import to attach
// Sangan, Dark Contract with the Witch to destroy and D/D Lance Soldiers to activate against.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (72181263, PZONE),
    (987311, MZONE),
    (987311, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (67322708, MZONE | OPPONENT),
    (67322708, MZONE | OPPONENT),
    (9765723, HAND),
    (9765723, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_MZONE);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, attcon);
    e2.set_target(duel, atttg);
    e2.set_operation(duel, attop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 3));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_target(duel, pentg);
    e3.set_operation(duel, penop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    c.is_faceup(duel) && c.is_set_card_full(duel, &[SET_DDD], Some(sc), SUMMON_TYPE_XYZ as u64, tp)
}

fn penfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) && !c.is_forbidden(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && penfilter(chkc, duel));
    }
    let pc = duel.get_field_group(tp, LOCATION_PZONE, 0).difference(c.into()).get_first();
    if chk == 0 {
        return Ok(pc.is_some_and(|pc| {
            duel.get_location_count(tp, LOCATION_MZONE) > 0
                && pc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
                && duel.is_existing_target(penfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
        }));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
    let g = duel.select_target(tp, penfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, pc.map(Into::into), 1, tp, 0);
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let pc = duel.get_field_group(tp, LOCATION_PZONE, 0).difference(c.into()).get_first();
    let Some(pc) = pc else { return Ok(()) };
    if duel.special_summon(pc, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.check_pendulum_zones(tp) {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn attcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    a.e.get_handler(duel).get_flag_effect(duel, ID) == 0
        && a.rp == 1 - a.tp
        && rc.is_original_type(duel, TYPE_MONSTER)
        && rc.is_on_field(duel)
        && rc.is_relate_to_effect(duel, re)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_CONTRACT])
}

fn atttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_overlay_count(duel) >= 2 || duel.is_existing_matching_card(desfilter, a.tp, LOCATION_ONFIELD, 0, 1, None));
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn attop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let b1 = c.is_relate_to_effect(duel, e) && c.get_overlay_count(duel) >= 2;
    let b2 = duel.is_existing_matching_card(desfilter, tp, LOCATION_ONFIELD, 0, 1, None);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (b2, aux::stringid(ID, 5))])?;
    let success = if op == Some(1) {
        c.remove_overlay_card(duel, tp, 2, 2, REASON_EFFECT)? == 2
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = duel.select_matching_card(tp, desfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        !dg.is_empty() && duel.destroy(dg, REASON_EFFECT)? > 0
    };
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if success
        && rc.is_relate_to_effect(duel, re)
        && rc.is_controler(duel, 1 - tp)
        && c.is_relate_to_effect(duel, e)
        && c.is_type(duel, TYPE_XYZ)
        && !rc.is_immune_to_effect(duel, e)
    {
        duel.overlay_full(c, rc, true)?;
    }
    Ok(())
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.e.get_handler(duel).is_type(duel, TYPE_PENDULUM) && duel.check_pendulum_zones(a.tp))
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
