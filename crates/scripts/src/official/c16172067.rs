// Tyrant Red Dragon Archfiend
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 16172067;

// In the Extra Deck, Synchro Summoned with two Ash Blossom & Joyous Spring (Level 3 Tuners) and Celtic Guardian (Level 4), with
// Mirror Force and Sakuretsu Armor on both sides, which only activate on an attack, to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (14558127, MZONE),
    (14558127, MZONE),
    (91152256, MZONE),
    (44095762, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
    (56120475, SZONE | OPPONENT),
    (56120475, SZONE | OPPONENT),
    (44095762, SZONE),
    (56120475, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN1: usize = 2;
    const MAX1: usize = 2;
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_MULTIPLE_TUNERS);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e1.set_label(duel, c.get_field_id(duel).into());
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_battle_phase()
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        // Duel.Destroy returns a number, which is always truthy in Lua.
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            e1.set_value(duel, 500);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
