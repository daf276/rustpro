// Hop Ear Squadron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 1980574;

// In the hand, with Stardust Synchron (a Level 4 Tuner) on the field and Stardust Charge Warrior (Level 6) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (37799519, MZONE), (64880894, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(64880894)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, sccon);
    e1.set_target(duel, sctg);
    e1.set_operation(duel, scop);
    c.register_effect(duel, e1);
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_CHAINING) && duel.is_turn_player(1 - a.tp) && duel.is_main_phase()
}

fn scfilter1(c: CardId, duel: &Duel, tp: u8, mc: CardId) -> bool {
    let mg = CardSet::from_iter([c, mc]);
    c.is_faceup(duel) && duel.is_existing_matching_card(|sc, duel| scfilter2(sc, duel, tp, mg), tp, LOCATION_EXTRA, 0, 1, None)
}

fn scfilter2(c: CardId, duel: &Duel, tp: u8, mg: CardSet) -> bool {
    duel.get_location_count_from_ex(tp, tp, Some(mg), c) > 0 && c.is_synchro_summonable(duel, None, Some(mg))
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && scfilter1(chkc, duel, tp, c));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(|tc, duel| scfilter1(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    duel.select_target(tp, |tc, duel| scfilter1(tc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) {
        return Ok(());
    }
    let mg = CardSet::from_iter([c, tc]);
    let g = duel.get_matching_group(|sc, duel| scfilter2(sc, duel, tp, mg), tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), Some(mg))?;
        }
    }
    Ok(())
}
