// Bramble Rose Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};
use engine::constants::MSG_DAMAGE;

const ID: u32 = 6560411;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PLANT, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_ROSE_DRAGON];
// Synchro Summoned with Angel Trumpeter (a Plant Tuner) and Green Phantom King (a Level 3 Normal Monster), with Black Rose
// Dragon in the Extra Deck and a second Angel Trumpeter in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_BLACK_ROSE_DRAGON, EXTRA),
    (87979586, MZONE),
    (22910685, MZONE),
    (87979586, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Message(MSG_DAMAGE),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1a.set_code(duel, EVENT_CHAINING);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1b.set_code(duel, EVENT_CHAIN_SOLVED);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && a.e.get_handler(duel).has_flag_effect(duel, 1)
    });
    e1b.set_operation(duel, damop);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, exsptg);
    e2.set_operation(duel, exspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, gysptg);
    e3.set_operation(duel, gyspop);
    c.register_effect(duel, e3);
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_CARD, 0, ID.into());
    if duel.damage(1 - tp, 600, REASON_EFFECT)? > 0 {
        for tc in duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None) {
            tc.update_attack(duel, -600, None, Some(c), None);
        }
    }
    Ok(())
}

fn exspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_code(duel, CARD_BLACK_ROSE_DRAGON)
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
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, None);
    if duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_race(duel, RACE_PLANT)
        && c.is_type(duel, TYPE_TUNER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}
