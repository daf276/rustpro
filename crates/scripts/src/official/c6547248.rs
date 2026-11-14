// Clown Crew New Face
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, REMOVED, SZONE};

const ID: u32 = 6547248;

pub const LISTED_SERIES: &[u16] = &[SET_CLOWN_CREW];
// Set and in the hand, with three Clown Crew cards in the Graveyard and banishment to shuffle, and Clown Crew Biancaviso in the
// hand to Tribute Summon by Tributing the Clown Crew Flair on the field, destroying the Set card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (82159583, GRAVE),
    (42759961, GRAVE),
    (20448151, REMOVED),
    (82159583, HAND),
    (42759961, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, _| duel.is_standby_phase_full(None));
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    e1.set_hint_timing(duel, TIMING_STANDBY_PHASE, TIMING_STANDBY_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition_mut(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e3.set_target(duel, pltg);
    e3.set_operation(duel, plop);
    c.register_effect(duel, e3);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_player_can_draw(tp, 0) && duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 3, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let filter = aux::necro_valley_filter(tdfilter);
    if !duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 3, None) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 3, 3, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 {
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
            duel.shuffle_deck(tp)?;
        }
        if duel.is_player_can_draw(tp, 0) {
            duel.break_effect()?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(ec) = a.eg.and_then(|eg| eg.get_first()) else {
        return false;
    };
    if !(ec.is_summon_player(duel, tp) && ec.is_tribute_summoned(duel)) {
        return false;
    }
    let mat_count = ec.get_material_count(duel) as i64;
    if mat_count > a.e.get_label(duel) {
        a.e.set_label(duel, mat_count);
    }
    mat_count > 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel) && chkc != c);
    }
    if chk == 0 {
        let mat_count = e.get_label(duel);
        e.set_label(duel, 0);
        c.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1, mat_count, 0);
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let max_count = c.get_flag_effect_label(duel, ID).unwrap_or(0);
    c.reset_flag_effect(duel, ID);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(
        tp,
        CardId::is_spell_trap,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        max_count as u8,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_location_count(a.tp, LOCATION_SZONE) > 0);
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
