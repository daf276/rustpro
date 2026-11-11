// Ice Ryzeal
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 8633261;

pub const LISTED_NAMES: &[u32] = &[ID];

// Two in the hand, Special Summoned by sending a card to the Graveyard or Normal Summoned, with Ryzeal monsters in the
// Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (35844557, DECK),
    (72238166, DECK),
    (34022970, DECK),
    (84433129, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cost_group(duel: &Duel, tp: u8, c: CardId) -> CardSet {
    duel.get_matching_group(
        CardId::is_able_to_grave_as_cost,
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        Some(c.into()),
    )
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let g = cost_group(duel, tp, c);
    !g.is_empty() && duel.get_mzone_count(tp, Some(g)) > 0
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let rg = cost_group(duel, tp, c);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_TOGRAVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    if let Some(c) = a.c {
        extra_deck_lock(duel, c, a.tp, aux::stringid(ID, 2));
    }
    Ok(())
}

// Shared by the Ryzeal monsters that Special Summon themselves: only Rank 4 Xyz Monsters come from the Extra Deck.
pub fn extra_deck_lock(duel: &mut Duel, c: CardId, tp: u8, description: u64) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, description);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_type(duel, TYPE_XYZ) && c.is_rank(duel, &[4]))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| {
        !(c.is_original_type(duel, TYPE_XYZ) && c.is_original_rank(duel, 4))
    });
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
