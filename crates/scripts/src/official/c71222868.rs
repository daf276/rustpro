// Raidraptor - Rising Rebellion Falcon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 71222868;

pub const LISTED_SERIES: &[u16] = &[SET_RAIDRAPTOR];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_WINGEDBEAST, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(13);
    const CT: usize = 5;
}

// Raidraptor - Force Strix in the Graveyard to copy; no Level 13 materials exist, so its Xyz Summon trigger is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (73347079, GRAVE),
    (73347079, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, Cost::detach_from_self::<3, 3>);
    e3.set_target(duel, copytg);
    e3.set_operation(duel, copyop);
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 0);
    Ok(false)
}

fn dmgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) && c.get_base_attack(duel) > 0
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_RAIDRAPTOR])
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let og = duel.get_operated_group().filter(duel, dmgfilter, None);
    if og.is_empty() {
        return Ok(());
    }
    let dam = og.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel)));
    let mg = c.get_overlay_group(duel).filter(duel, matfilter, None);
    if mg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3 {
        duel.break_effect()?;
        duel.damage(1 - tp, dam as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn copytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && matfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(matfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, matfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn copyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let code = tc.get_original_code_rule(duel);
        c.copy_effect(duel, code, RESETS_STANDARD_PHASE_END, 1);
    }
    Ok(())
}
