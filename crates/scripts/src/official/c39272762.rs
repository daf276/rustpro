// Neo Galaxy-Eyes Photon Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 39272762;

pub const LISTED_NAMES: &[u32] = &[CARD_GALAXYEYES_P_DRAGON];

// In the Extra Deck, Xyz Summoned from Galaxy-Eyes Photon Dragons and Blue-Eyes White Dragons; Number 19 (Freezadon, no activated effect) holds materials to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (ID, MZONE),
    (55067058, MZONE | OPPONENT),
    (55067058, MZONE | OPPONENT),
    (55067058, MZONE | OPPONENT),
    (55067058, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, negcon);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, attg);
    e3.set_operation(duel, atop);
    c.register_effect(duel, e3);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let found = c
        .get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_code(duel, CARD_GALAXYEYES_P_DRAGON), 1, None);
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, i64::from(found));
    }
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel) && a.e.get_label(duel) == 1
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e2);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_overlay_count(a.tp, 0, 1) != 0);
    }
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_overlay_group(a.tp, 0, 1);
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_grave(g, REASON_EFFECT)?;
    duel.break_effect()?;
    let c = a.e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, g.len() as i32 * 500);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_value(duel, g.len() as i32 - 1);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e2);
    Ok(())
}
