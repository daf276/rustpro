// Hi-Speedroid Cork Shooter
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 23361526;

pub const LISTED_SERIES: &[u16] = &[SET_SPEEDROID];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Speedroid Gum Prize (a Level 1 Tuner) and Lala Li-oon (Level 2), with Speedroid Scratch (a Speedroid Spell) to search.
// No Speedroid non-Tuner of Level 2 is ported, so a Synchro Summon from only Speedroid materials is out of reach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (70939418, MZONE),
    (70939418, MZONE),
    (9430387, MZONE),
    (9430387, MZONE),
    (29114773, DECK),
    (29114773, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, syncon);
    e1.set_cost(duel, syncost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, syncost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_attribute(duel, ATTRIBUTE_WIND)),
    );
}

fn syncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn syncost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_attribute(duel, ATTRIBUTE_WIND));
    duel.register_effect(e1, tp);
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPEEDROID]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label(duel) == 2
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sync: CardId) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason(duel) & (REASON_SYNCHRO | REASON_MATERIAL) == (REASON_SYNCHRO | REASON_MATERIAL)
        && c.get_reason_card(duel) == Some(sync)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mg = c.get_material(duel);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(!mg.is_empty()
            && ft >= mg.len() as i32
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && mg.filter_count(duel, |mc, duel| spfilter(mc, duel, e, tp, c), None) == mg.len());
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(mg), mg.len() as u8, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mg = c.get_material(duel);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    if mg.len() as i32 <= ft && mg.filter_count(duel, aux::necro_valley_filter(|mc, duel| spfilter(mc, duel, e, tp, c)), None) == mg.len() {
        duel.special_summon(mg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let ct = g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_SPEEDROID]), None);
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, if ct == g.len() { 2 } else { 1 });
    }
}
