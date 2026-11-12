// Assault Blackwing - Onimaru the Divine Thunder
use super::c16051717::add_type_effects;
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 80773359;

// In the Extra Deck, Synchro Summoned from Kochi (a Level 4 Tuner), Zephyros and Bora (Level 4), with Blackwing monsters of other
// Levels in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (41902352, MZONE),
    (41902352, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (49003716, MZONE),
    (49003716, MZONE),
    (2009101, GRAVE),
    (11613567, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    c.enable_revive_limit(duel);
    add_type_effects(duel, c, tncon);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_LVCHANGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e4.set_target(duel, lvtg);
    e4.set_operation(duel, lvop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_code(duel, EFFECT_UPDATE_ATTACK);
    e5.set_condition(duel, condtion);
    e5.set_value(duel, 3000);
    c.register_effect(duel, e5);
}

fn tncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SYNCHRO && a.e.get_label(duel) == 1
}

fn lvfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING]) && c.has_level(duel) && c.get_level(duel) != lv
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lv = a.e.get_handler(duel).get_level(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel, lv));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| lvfilter(c, duel, lv), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| lvfilter(c, duel, lv), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, a.e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn condtion(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let mg = c.get_material(duel);
    let ph = duel.get_current_phase();
    (ph == PHASE_DAMAGE || ph == PHASE_DAMAGE_CAL)
        && duel.get_attacker() == Some(c)
        && c.get_summon_type(duel) == SUMMON_TYPE_SYNCHRO
        && !mg.is_empty()
        && !mg.is_exists(duel, |c, duel| !c.is_type(duel, TYPE_SYNCHRO), 1, None)
}
