// Gimmick Puppet Fantasix Machinix
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 76290637;

pub const LISTED_SERIES: &[u16] = &[SET_RANK_UP_MAGIC, SET_GIMMICK_PUPPET];
// In the Extra Deck, Xyz Summoned with two of four Level 8 monsters, with Rank-Up-Magic Spells in the Deck to add, and a copy in the
// Graveyard that is Special Summoned when the Extra Deck copy is.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (39111158, MZONE),
    (39111158, MZONE),
    (39674352, MZONE),
    (39674352, MZONE),
    (3298689, DECK),
    (62104532, DECK),
    (62104532, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RANK_UP_MAGIC]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_player_can_summon(tp)
            && duel.is_player_can_additional_summon(tp));
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
    if duel.is_player_can_summon(tp) && duel.is_player_can_additional_summon(tp) {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
        e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_MACHINE));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel) && c.is_summon_player(duel, tp)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, tp), 1, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok((duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff))
            || (duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
                && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff);
    let b2 = duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = if b1 && b2 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (b2, aux::stringid(ID, 5))])?
    } else {
        Some(if b1 { 1 } else { 2 })
    };
    let target_player = if op == Some(1) { tp } else { 1 - tp };
    if duel.special_summon(c, 0, tp, target_player, false, false, POS_FACEUP_DEFENSE)? > 0
        && duel.is_existing_matching_card(aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 6))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
