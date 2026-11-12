// Odd-Eyes Raging Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 86238081;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub const PENDULUM_LEVEL: u32 = 7;

// In the Pendulum Zone with Pendulum Monsters in the Deck to place, and on the field with a Pendulum Zone card to destroy when it is
// destroyed by the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (72181263, DECK),
    (72181263, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    // The Xyz Monster's flag needs an Xyz Summon with an Xyz Monster as material, which no ported card makes.
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    Pendulum::add_procedure_full(duel, c, false, None);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, pctg);
    e1.set_operation(duel, pcop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_EXTRA_ATTACK);
    e4.set_value(duel, 1);
    e4.set_condition(duel, effcon);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_DESTROY);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, effcon);
    e5.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e5.set_target(duel, destg);
    e5.set_operation(duel, desop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 2));
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_DESTROYED);
    e6.set_property(duel, EFFECT_FLAG_DELAY);
    e6.set_condition(duel, pencon);
    e6.set_target(duel, pentg);
    e6.set_operation(duel, penop);
    c.register_effect(duel, e6);
}

fn pcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn pctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.check_pendulum_zones(tp) && duel.is_existing_matching_card(pcfilter, tp, LOCATION_DECK, 0, 1, None))
}

fn pcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) || !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, pcfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel) && a.e.get_label(duel) == 1
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    c.register_flag_effect_full(
        duel,
        0,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 3),
    );
    Ok(())
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0);
    }
    let sg = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let sg = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    let ct = duel.destroy(sg, REASON_EFFECT)?;
    if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct * 200);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let xyz = c.get_material(duel).is_exists(
        duel,
        |mc, duel| mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, PLAYER_NONE),
        1,
        None,
    );
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, i64::from(xyz));
    }
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.check_pendulum_zones(a.tp))
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
