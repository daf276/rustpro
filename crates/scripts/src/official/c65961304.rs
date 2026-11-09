// Kewl Tune B2B
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 65961304;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck and on the field with Kewl Tune Mix and Track Maker to Synchro Summon it, a Tuner in the Graveyard
// to recover, and the opponent's monsters to chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (16509007, MZONE),
    (16509007, GRAVE),
    (42781164, MZONE),
    (14558127, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, 2);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_TUNER));
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 2, ID);
    e3.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    e3.set_target(duel, thsptg);
    e3.set_operation(duel, thspop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_MATERIAL_CHECK);
    e4.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e4);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_type(duel, TYPE_TUNER)
        && !c.is_level(duel, &[10])
        && c.is_faceup(duel)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, mmz_chk);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    }
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let check = |tc: CardId, duel: &Duel| {
        duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    };
    let oper = |duel: &mut Duel| Ok(duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0);
    if !aux::to_hand_or_else_result(duel, tc, tp, check, oper, aux::stringid(ID, 2))? {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| c.is_synchro_summonable(duel, None, None), tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, None, None)?;
        }
    }
    Ok(())
}

// Shared by the Kewl Tune Synchro Monsters that can make two Tuners count as the Tuner.
pub fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let tuner = |mc: CardId, duel: &Duel| mc.is_type(duel, TYPE_TUNER) || mc.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some();
    if c.get_material(duel).is_exists(duel, tuner, 2, None) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_MULTIPLE_TUNERS);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD) | RESET_PHASE | PHASE_END);
        c.register_effect(duel, e1);
    }
}
