// Vagnawa the Moon-Eating Dragon
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 9839115;

// In the Extra Deck, Synchro Summoned with Junk Synchron (a Level 3 Tuner) and Kojikocy (Level 4).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (63977008, MZONE),
    (63977008, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_label(duel, 0);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    e0.set_operation(duel, matop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
}

const SYNCHRO_MATERIAL: u64 = SUMMON_TYPE_SYNCHRO as u64 | MATERIAL_SYNCHRO;

fn puretuner(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    c.is_has_effect(duel, EFFECT_NONTUNER).is_none() && c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp)
}

fn othertuner(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    (c.is_has_effect(duel, EFFECT_NONTUNER).is_some() && c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp))
        || c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some()
}

fn set_levels(duel: &mut Duel, e: EffectId, c: CardId, tuner: CardId) {
    let tunerlv = i64::from(tuner.get_synchro_level(duel, c));
    e.set_labels(duel, &[tunerlv, i64::from(c.get_level(duel)) - tunerlv]);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    if mg.is_empty() {
        e.set_label(duel, 0);
        return;
    }
    let tp = e.get_handler_player(duel);
    if let Some(tuner) = mg.filter(duel, |mc, duel| puretuner(mc, duel, c, tp), None).get_first() {
        e.set_label_object_group(duel, None);
        set_levels(duel, e, c, tuner);
        return;
    }
    let other_tuner_g = mg.filter(duel, |mc, duel| othertuner(mc, duel, c, tp), None);
    if other_tuner_g.len() > 1 {
        e.set_label_object_group(duel, Some(other_tuner_g));
    } else {
        e.set_label_object_group(duel, None);
        // An empty group is a nil index in the Lua.
        unported!(duel, other_tuner_g.is_empty());
        if let Some(tuner) = other_tuner_g.get_first() {
            set_levels(duel, e, c, tuner);
        }
    }
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, Some(c)) = (a.e, a.c) else { return Ok(()) };
    let Some(g) = e.get_label_object_group(duel) else { return Ok(()) };
    let tp = e.get_handler_player(duel);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    if let Some(tuner) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.hint_selection_full(tuner, true);
        set_levels(duel, e, c, tuner);
    }
    Ok(())
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) > 0));
    }
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(e.get_handler(duel).into()), 1, tp, 300);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 1, 1 - tp, 300);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(&[tuner_lv, nontuner_lv]) = e.get_label_object(duel).map(|e0| e0.get_labels(duel)) else {
        return Ok(());
    };
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, (nontuner_lv * 300) as i32);
        e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        c.register_effect(duel, e1);
        duel.break_effect()?;
        duel.damage(1 - tp, (tuner_lv * 300) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
