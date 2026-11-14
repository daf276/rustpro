// Raidraptor - Satellite Cannon Falcon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 23603403;

pub const LISTED_SERIES: &[u16] = &[SET_RAIDRAPTOR];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_WINGEDBEAST, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

// Raidraptor - Force Strix in the Graveyard to count. No Level 8 Winged Beasts exist for an Xyz Summon, so its trigger is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (73347079, GRAVE),
    (73347079, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, aux::stat_change_damage_step_condition);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel) && a.e.get_label(duel) == 1
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_chain_limit(chainlm, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(desfilter, a.tp, 0, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn chainlm(_: &Duel, _: EffectId, rp: u8, tp: u8, _: i64) -> bool {
    tp == rp
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let label = i64::from(g.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_RAIDRAPTOR]), 1, None));
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, label);
    }
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RAIDRAPTOR]) && c.is_monster(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(atkfilter, tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let ct = duel.get_matching_group_count(atkfilter, tp, LOCATION_GRAVE, 0, None) as i32;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, ct * -800);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
