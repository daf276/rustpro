// Crystron Quan
use super::c20050865::{scfilter1, scfilter2};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 93665266;

// On the field with Overdrive (a Level 4 Machine Normal Monster) in the hand and Accel Synchron (Level 5) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (2311603, HAND), (2311603, HAND), (37675907, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(37675907)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
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

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|tc, duel| scfilter1(tc, duel, e, tp, c), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |tc, duel| scfilter1(tc, duel, e, tp, c), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if !duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
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
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, None, Some(mg))?;
        }
    }
    Ok(())
}
