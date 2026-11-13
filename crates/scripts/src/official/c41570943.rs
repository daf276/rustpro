// Azure Ocean Dragon Lord - Neo-Daedalus Rage
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 41570943;

pub const LISTED_NAMES: &[u32] = &[CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, CARD_UMI];

// Link Summoned from four Normal Monsters (three with Atlantis, City of the Sea Dragon in the hand), with Levia-Dragon of
// Atlantis - Daedalus in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (64603351, GRAVE),
    (64603351, GRAVE),
    (38391684, HAND),
    (38391684, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 4;
    const MAX: Option<usize> = Some(4);
}

struct AtlantisProcedure;

impl LinkProcedure for AtlantisProcedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn spcon(duel: &Duel, e: EffectId, tp: u8) -> bool {
        splimit(duel, e, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    Link::add_procedure_full::<AtlantisProcedure>(duel, c, Some(aux::stringid(CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, 0)));
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_CODE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, CARD_UMI as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, tgcost);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

pub fn splimit(duel: &Duel, e: EffectId, tp: u8) -> bool {
    if duel
        .is_player_affected_by_effect(tp, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON)
        .is_some()
    {
        let c = e.get_handler(duel);
        c.assume_property(duel, ASSUME_LINK, u64::from(c.get_link(duel) - 1));
        return true;
    }
    false
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.lists_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_free_linked_zone(duel);
    if chk == 0 {
        return Ok(zone > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = c.get_free_linked_zone(duel);
    if c.is_relate_to_effect(duel, e) && zone > 0 {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_GRAVE, 0, None);
        let mut ct = 3.min(g.len()).min((zone & 0x1f).count_ones() as usize);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() && ct > 1 {
            ct = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, ct as u8, None)?;
        if !sg.is_empty() {
            duel.special_summon_full(sg, 0, tp, tp, false, false, POS_FACEUP, zone)?;
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn tgcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_UMI) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, tgcostfilter, LOCATION_ONFIELD)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    !(c.is_code(duel, CARD_UMI) && c.is_faceup(duel)) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(tgfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(tgfilter, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
