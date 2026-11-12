// Gold Pride - Star Leon
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 22390469;
const GOLD_PRIDE_LEON: u32 = 23512906;

pub const LISTED_NAMES: &[u32] = &[GOLD_PRIDE_LEON];

// In the Extra Deck, Synchro Summoned with Gold Pride - Leon (a Level 3 Tuner) and Bolt Penguin (Level 3), with Leon in the Deck to
// Special Summon and an opponent's monster (Sangan) to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (GOLD_PRIDE_LEON, MZONE),
    (GOLD_PRIDE_LEON, MZONE),
    (48531733, MZONE),
    (48531733, MZONE),
    (GOLD_PRIDE_LEON, DECK),
    (GOLD_PRIDE_LEON, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(GOLD_PRIDE_LEON),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop::<GOLD_PRIDE_LEON>);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_base_attack(duel) > 0
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && cfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(cfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, cfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let c = e.get_handler(duel);
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_base_attack(duel));
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(c.into()), 1, 0, atk);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let atk = tc.get_base_attack(duel);
    if c.update_attack(duel, atk, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None) == atk
        && duel.get_lp(tp) < duel.get_lp(1 - tp)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spfilter<const CODE: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CODE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

pub fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && c.is_able_to_extra(duel)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_EXTRA)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter::<CODE>(c, duel, e, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
