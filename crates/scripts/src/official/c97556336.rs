// Medius the Pure
use crate::prelude::*;

const ID: u32 = 97556336;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sp_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (sp_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let sp_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, sp_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, sp_chk);
    let Some(tc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        tc,
        tp,
        |_, duel| sp_chk && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(|_| ()),
        aux::stringid(ID, 3),
    )
}

fn tdfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    ((c.is_location(duel, LOCATION_HAND) && c.is_monster(duel)) || c.is_faceup(duel))
        && c.is_able_to_deck(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|sc, duel| tdfilter(sc, duel, tp), tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let filter = |sc, duel: &Duel| tdfilter(sc, duel, tp);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if sc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, sc)?;
    } else {
        duel.hint_selection(sc);
    }
    let c = e.get_handler(duel);
    if duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
        && c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
