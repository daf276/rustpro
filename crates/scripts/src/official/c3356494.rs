// Galaxy-Eyes Solflare Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 3356494;

pub const LISTED_NAMES: &[u32] = &[CARD_GALAXYEYES_P_DRAGON];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_attack_above(duel, 2000), 1, None)
    }
}

// Copies in the Extra Deck, Link Summoned from Galaxy Soldier (ATK 2000) and Photon Delta Wing, with both in the hand to discard, Photon Delta Wing in the
// Graveyard to add, and the opponent's Monster Reborn and Zombies in its Graveyard to make Special Summoned targets.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (46659709, MZONE),
    (47051709, MZONE),
    (47051709, MZONE),
    (46659709, HAND),
    (46659709, HAND),
    (47051709, HAND),
    (47051709, HAND),
    (47051709, GRAVE),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (21239280, GRAVE | OPPONENT),
    (21239280, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_PHOTON, SET_GALAXY]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sg = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, sg, sg.map_or(0, |sg| sg.len() as u8), 0, 0);
    Ok(false)
}

fn descfilter(c: CardId, duel: &Duel) -> bool {
    c.is_discardable(duel) && c.is_set_card(duel, &[SET_PHOTON, SET_GALAXY])
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let has = |setcode| sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[setcode]), 1, None);
    (has(SET_PHOTON) && has(SET_GALAXY), false)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(descfilter, tp, LOCATION_HAND, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 1, 2, &|duel, sg, _, _| rescon(duel, sg)));
    }
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| rescon(duel, sg);
    let sg = aux::select_unselect_group_full(duel, g, 1, 2, &rescon, tp, HINTMSG_DISCARD, Some(&rescon), false)?;
    duel.send_to_grave(sg, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_special_summoned(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_special_summoned, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_special_summoned, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}
