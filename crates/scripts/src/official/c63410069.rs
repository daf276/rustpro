// Desert Locusts
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 63410069;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Ash Blossom (a Level 3 Tuner) and Swordsman of Landstar (Level 3), and used as the Level 6
// of a Level 8 Synchro Monster (Ancient Fairy Life Dragon) with Gusto Squirro (a Level 2 Tuner).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (43321985, EXTRA),
    (43321985, EXTRA),
    (14558127, MZONE),
    (14558127, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (336369, MZONE),
    (336369, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e2.set_target(duel, sctarg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
}

fn tgtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let cp = duel.get_turn_player();
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, cp, 1);
    Ok(false)
}

fn tgop(duel: &mut Duel, _: &Args) -> Script<()> {
    let cp = duel.get_turn_player();
    let g = duel.get_field_group(cp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, cp, HINTMSG_DISCARD);
    let sg = g.select(duel, cp, 1, 1, None)?;
    duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    Ok(())
}

fn sctarg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let scfilter = |tc: CardId, duel: &Duel| tc.is_synchro_summonable(duel, Some(c.into()), None);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0 && duel.is_existing_matching_card(scfilter, tp, LOCATION_EXTRA, 0, 1, None));
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
        |tc, duel| tc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), None)?;
        }
    }
    Ok(())
}
