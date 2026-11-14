// Elfnote June Pride
use crate::prelude::*;

const ID: u32 = 5559570;

pub const LISTED_SERIES: &[u16] = &[SET_ELFNOTE];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race(duel, RACE_SPELLCASTER))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_sequence(duel, 2));
    e1.set_target_card(duel, |duel, _, c| !c.is_sequence(duel, 2));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.is_main_phase() && a.e.get_handler(duel).is_sequence(duel, 2));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ELFNOTE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_extra(duel)
            && duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(
                |sc, duel| spfilter(sc, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, tp, 0);
    let locations = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_EXTRA))
    {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |sc, duel: &Duel| spfilter(sc, duel, e, tp);
    let g = duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, None);
    let mut ft = ft.min(g.len() as i32).min(3);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let rescon = aux::dpcheck(|c, duel| i64::from(c.get_location(duel)));
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
