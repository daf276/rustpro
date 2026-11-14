// Crystron Cluster
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53829527;

pub const LISTED_SERIES: &[u16] = &[SET_CRYSTRON];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in hand and the Spell & Trap Zone, Crystron Ametrix on the field for the second target, Crystron monsters in
// the Graveyard to shuffle, and face-up opponent monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (76359406, MZONE),
    (83443619, GRAVE),
    (66938505, GRAVE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_REMOVE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_player_card_target(duel, rmlimit);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn rmlimit(duel: &Duel, e: EffectId, c: CardId, _: u8, r: u32) -> bool {
    c.is_set_card(duel, &[SET_CRYSTRON])
        && c.is_on_field(duel)
        && c.is_faceup(duel)
        && c.is_controler(duel, e.get_handler_player(duel))
        && !c.is_immune_to_effect(duel, e)
        && r & REASON_EFFECT > 0
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CRYSTRON]) && c.is_faceup(duel) && c.is_able_to_deck(duel) && !c.is_code(duel, ID)
}

fn crystsyncfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_CRYSTRON]) && c.is_type(duel, TYPE_SYNCHRO)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let exc = (!c.is_status(duel, STATUS_EFFECT_ENABLED)).then_some(c);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && exc != Some(chkc));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, exc.map(Into::into))
                && duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None),
        );
    }
    let ct = if duel.is_existing_matching_card(crystsyncfilter, tp, LOCATION_MZONE, 0, 1, None) {
        2
    } else {
        1
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(tdfilter),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
