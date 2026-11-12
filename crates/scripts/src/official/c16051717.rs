// Assault Blackwing - Raikiri the Rain Shower
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::ConditionFn;

const ID: u32 = 16051717;

// On the field and in the Extra Deck (Synchro Summoned from Gale, a Level 3 Tuner, and Zephyros, Level 4) with other Blackwing
// monsters, against the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (2009101, MZONE),
    (2009101, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    c.enable_revive_limit(duel);
    add_type_effects(duel, c, tncon);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

// The "add type" effects every Assault Blackwing Synchro shares: the Tuner type if a Blackwing was used as material.
pub fn add_type_effects(duel: &mut Duel, c: CardId, condition: ConditionFn) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, tnop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let label = i64::from(
        c.get_material(duel)
            .is_exists(duel, |mc, duel| mc.is_set_card(duel, &[SET_BLACKWING]), 1, None),
    );
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, label);
    }
}

pub fn tncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label(duel) == 1
}

fn tnop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_ADD_TYPE);
    e1.set_value(duel, TYPE_TUNER as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}

fn blackwing(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BLACKWING])
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(blackwing, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
            && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let ct = duel.get_matching_group_count(blackwing, tp, LOCATION_MZONE, 0, Some(c.into()));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tg: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let g = tg.unwrap_or_default().filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
