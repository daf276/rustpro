// Ritual Beast Ulti-Reirautari
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, REMOVED};

const ID: u32 = 96113307;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Copies in the Extra Deck and on the field, with Ritual Beast Tamer Elder to Link Summon from, banished Spiritual Beast Rampengu to return and the
// opponent's Ash Blossom to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14513016, MZONE),
    (14513016, MZONE),
    (14513016, MZONE),
    (88123329, REMOVED),
    (88123329, REMOVED),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_RELEASE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 1);
    e1.set_player_card_target(duel, |duel, _, _, _, r| {
        r & REASON_COST > 0 && duel.get_reason_effect().is_some_and(|re| re.is_activated(duel))
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, rmvtg);
    e3.set_operation(duel, rmvop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_faceup(duel) && (c.is_able_to_hand(duel) || c.is_able_to_extra(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_target(tp, thfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    let category = if g.and_then(|g| g.first()).is_some_and(|tc| tc.is_type(duel, TYPE_EXTRA)) {
        CATEGORY_TOEXTRA
    } else {
        CATEGORY_TOHAND
    };
    duel.set_operation_info(0, category, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_summonable(duel, true, None)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND | LOCATION_EXTRA))
    {
        return Ok(());
    }
    if tc.is_location(duel, LOCATION_HAND) {
        duel.shuffle_hand(tp)?;
    }
    if duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
        let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if let Some(sc) = g.first() {
            duel.summon(tp, sc, true, None)?;
        }
    }
    Ok(())
}

fn rmvfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_able_to_remove(duel)
        && c.is_can_be_effect_target(duel, e)
        && ((c.is_faceup(duel) && c.is_set_card(duel, &[SET_RITUAL_BEAST])) || c.is_controler(duel, 1 - tp))
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1, false)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| rmvfilter(c, duel, e, tp), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| {
            rescon(duel, sg, tp)
        }));
    }
    let rg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp), tp, HINTMSG_REMOVE)?;
    duel.set_target_card(rg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), rg.len() as u8, 0, 0);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
