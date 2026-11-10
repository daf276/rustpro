// Clockwork Knight
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, SZONE};

const ID: u32 = 41739381;
const CLOCKWORK_NIGHT: u32 = 84797028;

pub const LISTED_NAMES: &[u32] = &[CLOCKWORK_NIGHT];

// In the Extra Deck, Link Summoned with Golgoil (a Machine with 1000 or less ATK), with Clockwork Night (a Continuous Spell) on the
// field to send and in the Deck to search, and Golgoil in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7526150, MZONE),
    (62327910, MZONE),
    (7526150, GRAVE),
    (CLOCKWORK_NIGHT, SZONE),
    (CLOCKWORK_NIGHT, DECK),
    (CLOCKWORK_NIGHT, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
            && c.is_race_full(duel, RACE_MACHINE, Some(lc), sumtype, tp)
            && c.get_base_attack(duel) <= 1000
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_continuous_spell(duel) && c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thcostfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, thcostfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CLOCKWORK_NIGHT) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn release_options(tp: u8, to_check: CardId, exception: Option<CardId>) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(to_check),
        exception,
        ..ReleaseOptions::new(tp)
    }
}

fn sptributefilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_releasable_by_effect(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, hc: CardId) -> bool {
    c.is_attack_below(duel, 1000)
        && c.is_race(duel, RACE_MACHINE)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && duel.check_release_group_full(tp, sptributefilter, 1, release_options(tp, c, Some(hc)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let hc = e.get_handler(duel);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp, hc);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then_some(c);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let g = duel.select_release_group_full(tp, sptributefilter, (1, 1), false, release_options(tp, tc, exc))?;
    if !g.is_empty() && duel.release(g, REASON_EFFECT)? > 0 && tc.is_relate_to_effect(duel, e) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
