// Ritual Beast Ulti-Kimunfalcos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, REMOVED};

const ID: u32 = 58811192;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_RITUAL_BEAST], Some(lc), sumtype, tp)
    }
}

// Copies on the field and in the Extra Deck, with Ritual Beast Tamer Elder to Link Summon from and Normal Summon from the hand, and banished Tamer Elder
// and Spiritual Beast Rampengu to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14513016, MZONE),
    (14513016, MZONE),
    (14513016, HAND),
    (88123329, HAND),
    (14513016, REMOVED),
    (88123329, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1a.set_target_card(duel, |duel, e, c| {
        c.is_set_card(duel, &[SET_RITUAL_BEAST]) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
    });
    e1a.set_value(duel, 600);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, nscost);
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, Cost::self_to_extra);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn nscostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn nscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, nscostfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(sc) = g.first() {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST_TAMER, SET_SPIRITUAL_BEAST])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && c.is_faceup(duel)
        && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let has = |setcode| sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[setcode]), 1, None);
    (has(SET_RITUAL_BEAST_TAMER) && has(SET_SPIRITUAL_BEAST), false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg)));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 0 && tg.len() as i32 > ft {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        // The Lua passes the filter before the player to FilterSelect, which errors and ends the operation.
        return Ok(());
    }
    duel.special_summon(tg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    Ok(())
}
