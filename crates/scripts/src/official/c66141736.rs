// Red Rising Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 66141736;

pub const LISTED_SERIES: &[u16] = &[SET_RESONATOR];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FIEND, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp)
    }
}

// In the Extra Deck, Synchro Summoned from Soul Resonator and Darkness Resonator (Level 3 Fiend Tuners) and Shapesnatch (Level 5), with Crimson
// Resonator in the Graveyard to Special Summon. No Level 1 Resonator is ported, so the Graveyard effect finds no pair.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62991792, MZONE),
    (83445539, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (34761841, GRAVE),
    (34761841, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(|duel, c| counterfilter(c, duel)));
}

fn is_dark_dragon_synchro(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_SYNCHRO) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_DRAGON)
}

fn counterfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_summon_location(duel, LOCATION_EXTRA) || is_dark_dragon_synchro(c, duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !is_dark_dragon_synchro(c, duel) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| {
        !(c.is_original_type(duel, TYPE_SYNCHRO) && c.is_original_attribute(duel, ATTRIBUTE_DARK) && c.is_original_race(duel, RACE_DRAGON))
    });
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RESONATOR]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RESONATOR]) && c.get_level(duel) == 1 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter2(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_target(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 2, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 2, 2, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 2, 0, 0);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if ft < 2 || g.len() != 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
