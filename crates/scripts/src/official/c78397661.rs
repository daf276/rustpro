// Ecclesia and the Dark Dragon
use crate::prelude::*;

const ID: u32 = 78397661;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, rmsptg);
    e1.set_operation(duel, rmspop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_ALBAZ) || (c.is_level_below(duel, 4) && c.lists_code(duel, CARD_ALBAZ)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rmsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn rmspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && aux::remove_until::<aux::DefaultFieldReturnOp>(duel, c, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?.is_some()
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|sc, duel| spfilter(sc, duel, e, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn fustdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_level(duel, &[8])
        && c.is_fusion_monster(duel)
        && c.is_faceup(duel)
        && c.is_able_to_extra(duel)
        && c.is_can_be_effect_target(duel, e)
}

fn fieldtdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_able_to_deck(duel) && c.is_can_be_effect_target(duel, e)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let c = e.get_handler(duel);
    let fus = |sc, duel: &Duel| fustdfilter(sc, duel, e);
    let field = |sc, duel: &Duel| fieldtdfilter(sc, duel, e);
    if chk == 0 {
        return Ok(c.is_able_to_extra(duel)
            && duel.is_existing_matching_card(fus, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, Some(c.into()))
            && duel.is_existing_matching_card(field, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let (mut fusiong, mut fieldg) = (CardSet::default(), CardSet::default());
    while fieldg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        fusiong = duel.select_matching_card(tp, fus, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, Some(c.into()))?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        fieldg = duel.select_matching_card_full(tp, field, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, true, None)?;
    }
    let tg = fusiong.union(fieldg);
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg.union(c.into())), 3, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    if c.is_relate_to_effect(duel, e) {
        tg.insert(c);
        tg.keep_matching(duel, |c, duel| c.is_able_to_deck(duel), None);
        if tg.len() == 3 {
            duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
