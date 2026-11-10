// Shooting Riser Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 68431965;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned from Kewl Tune Cue and Luster Dragon, with Luster Dragons in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16387555, MZONE),
    (11091375, MZONE),
    (11091375, DECK),
    (11091375, DECK),
    (11091375, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e2.set_target(duel, sctarg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_level_below(duel, lv.saturating_sub(1)) && c.is_able_to_grave(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let lv = a.e.get_handler(duel).get_level(duel);
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let lv = c.get_level(duel);
    let g = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    let sent = duel.send_to_grave(g, REASON_EFFECT)?;
    if sent as usize != g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) {
        return Ok(());
    }
    let lv = tc.get_level(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_value(duel, -(lv as i32));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_target_range(duel, 1, 0);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    e2.set_label(duel, i64::from(tc.get_code(duel)));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_TRIGGER);
    e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e3);
    Ok(())
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_code(duel, e.get_label(duel) as u32) && re.is_monster_effect(duel)
}

fn sctarg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0
            && duel.is_existing_matching_card(
                |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
                tp,
                LOCATION_EXTRA,
                0,
                1,
                None,
            ));
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_controler(duel, 1 - tp) || !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), None)?;
        }
    }
    Ok(())
}
