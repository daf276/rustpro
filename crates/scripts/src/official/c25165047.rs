// Life Stream Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 25165047;

pub const MATERIAL: &[u32] = &[2403771];
pub const LISTED_NAMES: &[u32] = &[2403771];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[2403771])
    }
}

// In the Extra Deck, Synchro Summoned with Water Spirit (Level 1) and Power Tool Dragon, with Equip Spells in the Graveyard to banish and
// the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (2403771, MZONE),
    (2403771, MZONE),
    (1435851, GRAVE),
    (1557499, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_operation(duel, lpop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_value(duel, Value::ChangeDamage(damval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_target(duel, desreptg);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_MATERIAL_CHECK);
    e5.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e5);
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.set_lp(a.tp, 4000);
    Ok(())
}

fn damval(_: &mut Duel, _: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT != 0 { 0 } else { val }
}

fn repfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_spell(duel) && c.is_able_to_remove_as_cost(duel)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && duel.is_existing_matching_card(repfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, repfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(true)
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_type(duel, TYPE_TUNER), 2, None)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_MULTIPLE_TUNERS);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD & !RESET_TOFIELD | RESET_PHASE | PHASE_END);
        c.register_effect(duel, e1);
    }
}
