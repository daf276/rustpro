// Kewl Tune RS
use super::c65961304::valcheck;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 15665977;

pub const LISTED_NAMES: &[u32] = &[89392810];
pub const MATERIAL: &[u32] = &[89392810];

// Kewl Tune RS and Remix share it: a Tuner substitute with the given code and 1+ Tuners.
pub struct Procedure<const SUBSTITUTE: u32>;

impl<const SUBSTITUTE: u32> SynchroProcedure for Procedure<SUBSTITUTE> {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(_: CardId, _: &Duel, _: CardId, _: u64, _: u8) -> bool {
        false
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp) || c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some()
    }

    fn sub1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[SUBSTITUTE])
    }
}

// In the Extra Deck, Synchro Summoned with Kewl Tune Reco and Mix on the field, a Tuner in the Graveyard to banish and the
// opponent's monsters to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (89392810, MZONE),
    (89392810, MZONE),
    (16509007, MZONE),
    (16509007, MZONE),
    (16509007, GRAVE),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure<89392810>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, _| {
            300 * duel.get_matching_group_count(|c, duel| c.is_type(duel, TYPE_TUNER), 0, LOCATION_GRAVE, LOCATION_GRAVE, None) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_LEVEL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.get_base_attack(duel) <= 1700);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, _| duel.is_main_phase());
    e3.set_cost(duel, discost);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_MATERIAL_CHECK);
    e4.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e4);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    // The Lua reads the undefined `c` for the controller check, which errors on any card on the field.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        tc.negate_effects(duel, e.get_handler(duel), RESET_PHASE | PHASE_END, true);
    }
    Ok(())
}
