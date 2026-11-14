// S-Force Nightcatcher
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 67216732;

pub const LISTED_SERIES: &[u16] = &[SET_S_FORCE];
// Link Summoned from S-Force Edge Razor and another monster, with S-Force monsters to Special Summon from the hand, Deck and Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (91864689, MZONE),
    (91864689, MZONE),
    (47226949, MZONE),
    (91864689, HAND),
    (91864689, DECK),
    (20515672, EXTRA),
    (4611341, EXTRA),
];
// The search needs an S-Force Spell or Trap in the Deck, and none is ported.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_S_FORCE], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, cost);
    e2.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, { LOCATION_DECK | LOCATION_EXTRA }>,
    );
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_S_FORCE]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_S_FORCE]) && c.is_able_to_remove_as_cost(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, hc: Option<CardId>) -> bool {
    if c.is_location(duel, LOCATION_DECK) && duel.get_mzone_count(tp, hc.map(CardSet::from)) <= 0 {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, hc.map(CardSet::from), c) <= 0 {
        return false;
    }
    c.is_set_card(duel, &[SET_S_FORCE]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_matching_card(
                |tc, duel| spfilter(tc, duel, e, tp, Some(c)),
                tp,
                LOCATION_DECK | LOCATION_EXTRA,
                0,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, spcostfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.remove(g.union(c.into()), POS_FACEUP, REASON_COST)?;
    // The handler is already banished here, so the Lua returns false to skip Cost.SelfBanish
    Ok(false)
}

fn extracon(duel: &Duel, _: EffectId, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.is_existing_matching_card(
        |tc, duel| spfilter(tc, duel, e, tp, Some(c)),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        a.c.map(Into::into),
    )
}

// Cost.AND(Cost.Replaceable(spcost, extracon), Cost.SelfBanish)
fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(Cost::replaceable_full(duel, a, 0, spcost, extracon)? && Cost::self_banish(duel, a, 0)?);
    }
    unported!(duel, duel.get_player_effect(a.tp, EFFECT_COST_REPLACE).into_iter().next().is_some());
    Cost::replaceable_full(duel, a, 1, spcost, extracon)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, None),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
