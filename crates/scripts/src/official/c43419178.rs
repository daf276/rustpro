// Ashtraya Mikumari-no-Mikoto
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 43419178;

pub const LISTED_SERIES: &[u16] = &[SET_ASHTRA];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp)
    }
}

// Synchro Summoned with Ashtra Divine Domain (a Tuner once Special Summoned) and Yoa, with Ashtra cards in the Deck to send, Traps
// in the Graveyard and banishment to Set, and the opponent's monsters to activate effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (37839434, HAND),
    (37839434, MZONE),
    (37839434, DECK),
    (37839434, GRAVE),
    (37839434, REMOVED),
    (37839434, SZONE),
    (1710647, MZONE),
    (1710647, DECK),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ASHTRA]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, tp, (LOCATION_DECK | LOCATION_ONFIELD) as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let tg = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if !tg.is_empty() {
            duel.hint_selection(tg);
            duel.send_to_grave(tg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel, allow_any_trap: bool) -> bool {
    c.is_trap(duel)
        && c.is_faceup(duel)
        && c.is_ssetable(duel)
        && (c.is_set_card(duel, &[SET_ASHTRA]) || (allow_any_trap && c.is_location(duel, LOCATION_GRAVE)))
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let allow_any_trap = duel.get_matching_group_count(CardId::is_facedown, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None) >= 3;
    let filter = |c: CardId, duel: &Duel| setfilter(c, duel, allow_any_trap);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SET, g, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.sset(a.tp, tc)?;
    }
    Ok(())
}
