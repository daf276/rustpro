// Accel Synchro Stardust Dragon
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 30983281;

pub const LISTED_NAMES: &[u32] = &[CARD_STARDUST_DRAGON];
// In the Extra Deck, Synchro Summoned with Mono Synchron and Ignihorse Dragonis, a Level 7 Normal Monster, with a second Mono
// Synchron in the Graveyard to Special Summon, a second copy in the Extra Deck, and Stardust Dragon there for the second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (56897896, MZONE),
    (28103028, MZONE),
    (56897896, GRAVE),
    (44508094, EXTRA),
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
    e1.set_count_limit(duel, 1, (ID, 1));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, gysptg);
    e1.set_operation(duel, gyspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, (ID, 2));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable_full(duel, a, chk, extra_deck_cost, extracon));
    e2.set_target(duel, exsptg);
    e2.set_operation(duel, exspop);
    c.register_effect(duel, e2);
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 2) && c.is_type(duel, TYPE_TUNER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}

fn exspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rc: Option<CardId>, step: bool) -> bool {
    let mg = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_can_be_synchro_material_full(duel, None, None, None)),
        tp,
        LOCATION_MZONE,
        0,
        rc.map(Into::into),
    );
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, Some(c), None, REASON_SYNCHRO);
    c.is_code(duel, CARD_STARDUST_DRAGON)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, rc.map(Into::into), c) > 0
        && (step
            || (pg.is_empty()
                && aux::check_summon_gate(duel, tp).is_none_or(|limit| limit >= 2)
                && duel.is_existing_matching_card(
                    |sc, duel| sc.is_synchro_summonable(duel, None, Some(mg.union(c.into()))),
                    tp,
                    LOCATION_EXTRA,
                    0,
                    1,
                    Some(c.into()),
                )))
}

fn exspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        let (e, tp) = (a.e, a.tp);
        let rc = Some(e.get_handler(duel));
        return Ok(duel.is_existing_matching_card(|c, duel| exspfilter(c, duel, e, tp, rc, false), tp, LOCATION_EXTRA, 0, 1, None));
    }
    Ok(false)
}

fn extracon(duel: &Duel, _: EffectId, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    duel.is_existing_matching_card(|c, duel| exspfilter(c, duel, e, tp, None, false), tp, LOCATION_EXTRA, 0, 1, None)
}

fn extra_deck_cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Cost::and(duel, a, chk, &[Cost::self_tribute, exspcost])
}

fn exsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn exspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| exspfilter(c, duel, e, tp, None, true),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    let Some(stardustc) = g.get_first() else { return Ok(()) };
    stardustc.set_material(duel, None);
    if duel.special_summon(stardustc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0 {
        stardustc.complete_procedure(duel);
        let mut sg = CardSet::from(stardustc);
        if duel.is_existing_matching_card(
            |sc, duel| sc.is_synchro_summonable(duel, None, None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(
                tp,
                |sc, duel| sc.is_synchro_summonable(duel, None, None),
                tp,
                LOCATION_EXTRA,
                0,
                1,
                1,
                None,
            )?;
            if let Some(sync) = g.get_first() {
                sg.insert(sync);
                duel.synchro_summon(tp, sync, None, None)?;
            }
        }
        let c = e.get_handler(duel);
        for sc in sg.iter() {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 2));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
            e1.set_value(
                duel,
                Value::Effect(|duel, e, te| te.get_owner_player(duel) == 1 - e.get_handler_player(duel) && te.is_activated(duel)),
            );
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END & !RESET_TOFIELD);
            sc.register_effect_full(duel, e1, true);
        }
    }
    Ok(())
}
