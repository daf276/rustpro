// Crimson Resonator
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 34761841;

pub const LISTED_SERIES: &[u16] = &[SET_RESONATOR];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand to be Normal Summoned beside Red Dragon Archfiend with Vision Resonator to Special Summon, or after the opponent's Raigeki empties the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (98396890, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, spcost);
    e2.set_condition(duel, spcon2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(|duel, c| counterfilter(c, duel)));
}

fn darksynchrodragon(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_SYNCHRO) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_DRAGON)
}

fn counterfilter(c: CardId, duel: &Duel) -> bool {
    c.get_summon_location(duel) != LOCATION_EXTRA || darksynchrodragon(c, duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !darksynchrodragon(c, duel) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| {
        !(c.is_original_type(duel, TYPE_SYNCHRO) && c.is_original_attribute(duel, ATTRIBUTE_DARK))
    });
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    Ok(false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && darksynchrodragon(c, duel) && c.is_type(duel, TYPE_SYNCHRO) && c.is_race(duel, RACE_DRAGON)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_field_group(tp, LOCATION_MZONE, 0).len() == 2
        && duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, 0, Some(a.e.get_handler(duel).into())) == 1
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RESONATOR]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(
            ft > 0 && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    ft = ft.min(2);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        ft as u8,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
