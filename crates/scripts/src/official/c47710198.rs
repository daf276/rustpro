// Swordsoul Sinister Sovereign - Qixing Longyuan
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 47710198;

// On the field with Strategist Longyuan and Mystery Shell Dragon (a Wyrm) in the hand to summon a Token and Synchro Summon
// Chengying (a Wyrm Synchro Monster) from the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (93490856, HAND),
    (93490856, HAND),
    (93490856, DECK),
    (18108166, HAND),
    (96633955, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_WYRM, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, drcon);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| {
        duel.get_current_phase() != PHASE_DAMAGE && duel.get_current_phase() != PHASE_DAMAGE_CAL
    });
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_operation(duel, regop);
    c.register_effect(duel, e2a);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, rmcon2);
    e3.set_target(duel, rmtg2);
    e3.set_operation(duel, rmop2);
    c.register_effect(duel, e3);
}

// The group e2 and e2a share as their label object.
fn group_key(duel: &Duel, e: EffectId) -> u64 {
    u64::from(ID) << 16 | e.get_owner(duel).index() as u64
}

fn label_group(duel: &Duel, e: EffectId) -> CardSet {
    duel.get_script_group(group_key(duel, e)).unwrap_or_default()
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_synchro_summoned(duel)
        && c.is_race(duel, RACE_WYRM)
        && c.is_summon_player(duel, tp)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, Some(c.into()))
}

fn filter(c: CardId, duel: &Duel, tp: u8, e: Option<EffectId>) -> bool {
    c.is_summon_player(duel, 1 - tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_able_to_remove(duel)
        && e.is_none_or(|e| c.is_relate_to_effect(duel, e))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = label_group(duel, e).filter(duel, |c, duel| filter(c, duel, tp, None), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1200);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = label_group(duel, e);
    if g.is_empty() {
        return Ok(());
    }
    let bg = g.filter_select(duel, tp, |c, duel| filter(c, duel, tp, Some(e)), 1, 1, None)?;
    if !bg.is_empty() && duel.remove(bg, POS_FACEUP, REASON_EFFECT)? > 0 {
        duel.damage(1 - tp, 1200, REASON_EFFECT)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_flag_effect(tp, ID) > 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, tp, None), None);
    if !tg.is_empty() {
        for tc in tg {
            tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
        }
        let mut g = if duel.get_current_chain() == 0 {
            CardSet::default()
        } else {
            label_group(duel, e)
        };
        g.merge(tg);
        let g = g.filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
        duel.set_script_group(group_key(duel, e), Some(g));
        duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}

fn rmcon2(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
        && a.re
            .is_some_and(|re| re.is_spell_trap_effect(duel) && re.get_handler(duel).is_relate_to_effect(duel, re))
}

fn rmtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    if chk == 0 {
        return Ok(re.get_handler(duel).is_able_to_remove(duel));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(re.get_handler(duel).into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 1200);
    Ok(false)
}

fn rmop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if !re.get_handler(duel).is_relate_to_effect(duel, re) {
        return Ok(());
    }
    if duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)? > 0 {
        duel.damage(1 - a.tp, 1200, REASON_EFFECT)?;
    }
    Ok(())
}
