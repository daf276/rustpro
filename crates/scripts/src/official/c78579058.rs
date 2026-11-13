// Levia-Dragoon - Daedalus
use super::c41570943::splimit;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 78579058;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON];

// Link Summoned from WATER Normal Monsters (Island Turtle), two with Atlantis, City of the Sea Dragon in the hand, with Levia-Dragon
// of Atlantis - Daedalus in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4042268, MZONE),
    (4042268, MZONE),
    (4042268, MZONE),
    (4042268, MZONE),
    (64603351, DECK),
    (64603351, DECK),
    (38391684, HAND),
    (38391684, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(lc), sumtype, tp)
    }
}

struct AtlantisProcedure;

impl LinkProcedure for AtlantisProcedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        Procedure::f(c, duel, lc, sumtype, tp)
    }

    fn spcon(duel: &Duel, e: EffectId, tp: u8) -> bool {
        splimit(duel, e, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    Link::add_procedure_full::<AtlantisProcedure>(duel, c, Some(aux::stringid(CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, 0)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, CARD_UMI as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, _| duel.is_main_phase_full(None));
    e3.set_cost(duel, discardcost);
    e3.set_target(duel, discardtg);
    e3.set_operation(duel, discardop);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.lists_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn discardcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_UMI) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn discardcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, discardcostfilter, LOCATION_ONFIELD)
}

fn discardtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    let ct = g.len();
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(1 - tp, ct));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, Some(g), ct as u8, 1 - tp, ct as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, ct as i32);
    duel.set_chain_limit(|_, _, _, _, _| false, 0);
    Ok(false)
}

fn discardop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if !g.is_empty() {
        let ct = duel.send_to_grave(g, REASON_DISCARD | REASON_EFFECT)?;
        if ct == 0 {
            return Ok(());
        }
        duel.draw(1 - tp, ct as u32, REASON_EFFECT)?;
    }
    Ok(())
}
