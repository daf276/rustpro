// Chaos End Ruler - Ruler of the Beginning and the End
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 41424250;

// In the hand with a LIGHT Warrior and a DARK Fiend in the Graveyard, then on the field with cards on the opponent's side to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (20721928, GRAVE),
    (12057781, GRAVE),
    (20721928, GRAVE),
    (12057781, GRAVE),
    (20721928, GRAVE | OPPONENT),
    (12057781, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    e0.set_value(duel, 9300);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_DISABLE_SPSUMMON);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2a.set_operation(duel, limop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2b.set_code(duel, EVENT_CHAIN_END);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e2b.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).reset_flag_effect(duel, ID);
        duel.set_chain_limit_till_chain_end(|_, _, _, _, _| false, 0);
        Ok(())
    });
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 9300])
    });
    e3.set_cost(duel, Cost::pay_lp::<1000>);
    e3.set_target(duel, bandamtg);
    e3.set_operation(duel, bandamop);
    e3.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_WARRIOR))
        || (c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_FIEND)))
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn propertyfilter(c: CardId, duel: &Duel, attribute: u32, race: u64) -> bool {
    c.is_attribute(duel, attribute) && c.is_race(duel, race)
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        (
            duel.get_mzone_count(tp, Some(sg)) > 0
                && sg.is_exists(duel, |c, duel| propertyfilter(c, duel, ATTRIBUTE_LIGHT, RACE_WARRIOR), 1, None)
                && sg.is_exists(duel, |c, duel| propertyfilter(c, duel, ATTRIBUTE_DARK, RACE_FIEND), 1, None),
            false,
        )
    }
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    g.len() >= 2 && duel.get_mzone_count(tp, Some(g)) > 0 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon(tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let sg = aux::select_unselect_group_full(duel, g, 2, 2, &rescon(tp), tp, HINTMSG_REMOVE, None, true)?;
    if sg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sg) = a.e.get_label_object_group(duel) {
        duel.remove(sg, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn limop(duel: &mut Duel, a: &Args) -> Script<()> {
    match duel.get_current_chain() {
        0 => duel.set_chain_limit_till_chain_end(|_, _, _, _, _| false, 0),
        1 => {
            a.e.get_handler(duel)
                .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
        _ => {}
    }
    Ok(())
}

fn bandamtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, None);
    let count = g.len();
    duel.set_operation_info(
        0,
        CATEGORY_REMOVE,
        Some(g),
        count as u8,
        tp,
        (LOCATION_ONFIELD | LOCATION_GRAVE) as i32,
    );
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, count as i32 * 500);
    Ok(false)
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_REMOVED) && !c.is_reason(duel, REASON_REDIRECT)
}

fn bandamop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, None);
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0 {
        let damage = 500 * duel.get_operated_group().filter_count(duel, damfilter, None) as u32;
        if damage > 0 {
            duel.damage(1 - tp, damage, REASON_EFFECT)?;
        }
    }
    Ok(())
}
