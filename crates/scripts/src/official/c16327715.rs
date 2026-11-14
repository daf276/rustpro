// Junk Warrior Extreme
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 16327715;

pub const LISTED_SERIES: &[u16] = &[SET_JUNK];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[CARD_JUNK_SYNCHRON]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[CARD_JUNK_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_SYNCHRON];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];
// In the Extra Deck, Synchro Summoned with Junk Synchron and Shapesnatch (a Level 5 Normal Monster), with Atlantean Pikeman (Level 2) in the
// GY to revive and a Junk Warrior in the Extra Deck to summon after battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_JUNK_SYNCHRON, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (26976414, GRAVE),
    (26976414, GRAVE),
    (CARD_JUNK_WARRIOR, EXTRA),
    (4035199, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, gysptg);
    e1.set_operation(duel, gyspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::bdocon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, exsptg);
    e2.set_operation(duel, exspop);
    c.register_effect(duel, e2);
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 2) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 0 {
        let g = duel.get_matching_group(|c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
        ft = ft.min(g.len() as i32);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let ft = ft as u8;
        let sg = duel.select_matching_card(tp, |c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, ft, ft, None)?;
        if !sg.is_empty() && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
            for sc in duel.get_operated_group() {
                let e1 = duel.create_effect(c);
                e1.set_description(duel, 3302);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                sc.register_effect(duel, e1);
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, e, _, sump, _, _, _, _| {
        i64::from(duel.get_activity_count(sump, ACTIVITY_SPSUMMON)) - e.get_label(duel) >= 1
    });
    let label = duel.get_activity_count(tp, ACTIVITY_SPSUMMON);
    e1.set_label(duel, label.into());
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_LEFT_SPSUMMON_COUNT);
    e2.set_value(duel, Value::SpSummonCount(countval));
    duel.register_effect(e2, tp);
    Ok(())
}

fn countval(duel: &Duel, e: EffectId, _: Option<EffectId>, tp: u8) -> i32 {
    let label = e.get_label(duel);
    let sp = i64::from(duel.get_activity_count(tp, ACTIVITY_SPSUMMON));
    if sp - label >= 1 { 0 } else { (1 - sp + label) as i32 }
}

fn exspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, Some(c), None, REASON_SYNCHRO);
    pg.is_empty()
        && c.is_set_card(duel, &[SET_JUNK])
        && c.is_type(duel, TYPE_SYNCHRO)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn exsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mc = e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(|c, duel| exspfilter(c, duel, e, tp, mc), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn exspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mc = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| exspfilter(c, duel, e, tp, mc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else {
        return Ok(());
    };
    sc.set_material(duel, None);
    if duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}
