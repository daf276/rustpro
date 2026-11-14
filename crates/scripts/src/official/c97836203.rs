// T.G. Halberd Cannon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 97836203;

pub const LISTED_SERIES: &[u16] = &[SET_TG];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with T.G. Mighty Striker (a Level 2 Tuner) and two T.G. Hyper Librarians, with T.G. Jet
// Falcon in the Graveyard to Special Summon, against the opponent's Raigeki and Summoned Skull.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (32480825, MZONE),
    (90953320, MZONE),
    (90953320, MZONE),
    (37300735, GRAVE),
    (12580477, HAND | OPPONENT),
    (70781052, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_SUMMON);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_condition(duel, |duel, _| duel.get_current_chain() == 0);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_code(duel, EVENT_FLIP_SUMMON);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, a.tp, false, true, POS_FACEUP)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TG]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
