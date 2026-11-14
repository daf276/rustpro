// Bloom Harmonist the Melodious Composer
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 34974462;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FAIRY, Some(lc), sumtype, tp)
    }
}

// Link Summoned from Fairy Normal Monsters, with Melodious monsters of Levels 4 and 5 (Aria, Elegy) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (1929294, MZONE),
    (27288416, MZONE),
    (27288416, MZONE),
    (40502912, DECK),
    (40502912, DECK),
    (79514956, DECK),
    (79514956, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(79514956)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, actcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_set_card(duel, &[SET_MELODIOUS])),
    );
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None)
                && duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0,
        );
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_MELODIOUS]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_MELODIOUS])
        && c.has_level(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, zone)
}

fn level_check() -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    aux::dpcheck(|c, duel| i64::from(c.get_level(duel)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let zone = e.get_handler(duel).get_linked_zone(duel) & ZONES_MMZ;
        let ct = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone);
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_DECK, 0, None);
        return Ok(ct > 1
            && g.len() >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 2, 2, &level_check()));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let zone = c.get_linked_zone(duel) & ZONES_MMZ;
    let ct = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone);
    if zone == 0 || ct < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &level_check(), tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon_full(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE, zone)?;
    }
    Ok(())
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker()
        .is_some_and(|ac| ac.is_set_card(duel, &[SET_MELODIOUS]) && a.e.get_handler(duel).get_linked_group(duel).is_contains(ac))
}
