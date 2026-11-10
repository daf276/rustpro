// Deep Sea Repetiteur
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 33467872;
const DEEP_SEA_DIVA: u32 = 78868119;

pub const MATERIAL: &[u32] = &[DEEP_SEA_DIVA];
pub const LISTED_NAMES: &[u32] = &[DEEP_SEA_DIVA];

// In the Extra Deck, Synchro Summoned with Deep Sea Diva and Flame Manipulator (Level 3), with an Atlantean Pikeman (WATER) to discard in the hand
// and Frostosaurus (Level 6) in the Graveyard to revive when it is destroyed by the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (DEEP_SEA_DIVA, MZONE),
    (34460851, MZONE),
    (26976414, HAND),
    (6631034, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[DEEP_SEA_DIVA])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, atkcost);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atkcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_level_below(duel, 4) && c.is_discardable(duel)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(atkcfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, atkcfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 800);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_synchro_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER)
        && c.is_level_above(duel, 5)
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_target_range(duel, 1, 0);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_attribute(duel, ATTRIBUTE_WATER));
        duel.register_effect(e1, tp);
    }
    Ok(())
}
