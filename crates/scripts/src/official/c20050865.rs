// Crystron Citree
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 20050865;

// On the field with Cycroid (a Level 3 Machine Normal Monster) in the Graveyard and Accel Synchron (Level 5) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (45945685, GRAVE), (45945685, GRAVE), (37675907, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(37675907)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, sccon);
    e1.set_target(duel, sctg);
    e1.set_operation(duel, scop);
    c.register_effect(duel, e1);
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_CHAINING)
        && duel.is_turn_player(1 - a.tp)
        && (duel.is_main_phase() || duel.is_battle_phase())
}

// Shared with Crystron Rion.
pub fn scfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    let mg = CardSet::from_iter([c, mc]);
    !c.is_type(duel, TYPE_TUNER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|sc, duel| scfilter2(sc, duel, mg), tp, LOCATION_EXTRA, 0, 1, None)
}

pub fn scfilter2(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_synchro_summonable(duel, None, Some(mg))
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && scfilter1(chkc, duel, e, tp, c));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|tc, duel| scfilter1(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |tc, duel| scfilter1(tc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || !duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    tc.register_effect(duel, e2);
    duel.special_summon_complete()?;
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let mg = CardSet::from_iter([c, tc]);
    let g = duel.get_matching_group(|sc, duel| scfilter2(sc, duel, mg), tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
        let e2 = e1.clone_effect(duel);
        tc.register_effect_full(duel, e2, true);
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, None, Some(mg))?;
        }
    }
    Ok(())
}
