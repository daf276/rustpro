// Madolche Queen Tiarafraise
use super::c37164373::madolche_filter;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 49689480;
const TIARAMISU: u32 = 37164373;

pub const LISTED_NAMES: &[u32] = &[ID, TIARAMISU];

// In the Extra Deck, Xyz Summoned with Madolche Puddingcess or over Madolche Queen Tiaramisu, with Madolche Puddingcess in the Graveyard,
// the opponent's monsters to shuffle, and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (74641045, MZONE),
    (74641045, MZONE),
    (74641045, MZONE),
    (TIARAMISU, MZONE),
    (74641045, GRAVE),
    (74641045, GRAVE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(madolche_filter);
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, texcon);
    e2.set_target(duel, textg);
    e2.set_operation(duel, texop);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[TIARAMISU])
}

fn mdlchfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MADOLCHE]) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && mdlchfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(mdlchfilter, tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, mdlchfilter, tp, LOCATION_GRAVE, 0, 1, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() || duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let ct = duel.get_operated_group().filter_count(
        duel,
        |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA | LOCATION_HAND),
        None,
    );
    if ct > 0 && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, 0, LOCATION_ONFIELD, 1, None) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let rg = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
        if !rg.is_empty() {
            duel.hint_selection(rg);
            duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn texcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason_player(duel, 1 - tp) && c.is_previous_controler(duel, tp)
}

fn textg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn texop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
