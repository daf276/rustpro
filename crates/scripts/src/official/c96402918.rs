// Lancea, Ancestral Dragon of the Ice Mountain
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 96402918;

pub const LISTED_SERIES: &[u16] = &[SET_ICE_BARRIER];

const LOCATION_HAND_DECK_EXTRA_GRAVE: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE;

// In the Extra Deck, Synchro Summoned with Water Spirit (a WATER Tuner) and Phantom of Yubel (Level 9) or Guardragon Justicia and Evil HERO Dark Gaia (Level 8), with General Raiho in the Deck
// to Special Summon, Brionac in the Extra Deck to Special Summon when it leaves the field, and Dark Hole and Raigeki from the opponent to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (80453041, MZONE),
    (11012154, MZONE),
    (58332301, MZONE),
    (ID, MZONE),
    (81275309, DECK),
    (81275309, HAND),
    (50321796, EXTRA),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 2, ID);
    e1.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, exspcon);
    e2.set_target(duel, exsptg);
    e2.set_operation(duel, exspop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_ICE_BARRIER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND_DECK_EXTRA_GRAVE, 0, 1, None));
    }
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND_DECK_EXTRA_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, 1 - tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND_DECK_EXTRA_GRAVE, 0, 1, 1, None)?;
    if g.is_empty() || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let posfilter = |c: CardId, duel: &Duel| c.is_attack_pos(duel) && c.is_can_change_position(duel);
    if duel.is_existing_matching_card(posfilter, tp, 0, LOCATION_MZONE, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let posg = duel.select_matching_card(tp, posfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if posg.is_empty() {
            return Ok(());
        }
        duel.hint_selection_full(posg, true);
        duel.break_effect()?;
        duel.change_position(posg, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn exspcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.rp == 1 - tp
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
        && c.is_synchro_summoned(duel)
}

fn exspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ICE_BARRIER])
        && c.is_type(duel, TYPE_SYNCHRO)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn exsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| exspfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn exspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| exspfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}
