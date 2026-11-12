// Gaia Stream, the Graceful Force
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 13289758;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters (Frostosaurus, Air Eater, Boulder Tortoise), with the opponent's
// monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (6631034, MZONE),
    (8353769, MZONE),
    (9540040, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e0.set_condition(duel, matxyzcond);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_DAMAGE_STEP_END);
    e3.set_operation(duel, detachop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_rank(duel, &[5, 7]) && c.is_type_full(duel, TYPE_XYZ, Some(lc), SUMMON_TYPE_XYZ as u64, tp) && c.is_faceup(duel)
}

fn matxyzcond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_SPSUMMON_TURN) && c.is_xyz_summoned(duel)
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let g = e
        .get_handler(duel)
        .get_overlay_group(duel)
        .filter(duel, |c, duel| c.has_level(duel) || c.has_rank(duel), None);
    200 * (g.get_sum(duel, |c, duel| i64::from(c.get_level(duel))) + g.get_sum(duel, |c, duel| i64::from(c.get_rank(duel)))) as i32
}

fn detachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.get_overlay_count(duel) > 0 {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    }
    Ok(())
}
