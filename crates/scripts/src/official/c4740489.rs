// Magnetic Field
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4740489;

pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR];

// In the hand to activate, with the Normal Magnet Warriors on the field to battle and in the GY to summon, and the opponent's
// Sangan to battle with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (99785935, MZONE),
    (39256679, MZONE),
    (11549357, MZONE),
    (99785935, GRAVE),
    (11549357, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_DAMAGE_STEP_END);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition_mut(duel, atcon);
    e3.set_operation(duel, atop);
    c.register_effect(duel, e3);
}

fn rock_earth(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_attribute(duel, ATTRIBUTE_EARTH)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_below(duel, 4) && rock_earth(c, duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAGNET_WARRIOR]) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    stock::special_summon_first_target::<POS_FACEUP>(duel, a)
}

fn atcon(duel: &mut Duel, a: &Args) -> bool {
    let tp = a.tp;
    let (Some(at), Some(d)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    if d.is_controler(duel, tp) {
        a.e.set_label_object_group(duel, Some(at.into()));
        rock_earth(d, duel) && d.is_faceup(duel) && at.is_relate_to_battle(duel) && at.is_location(duel, LOCATION_ONFIELD)
    } else if at.is_controler(duel, tp) {
        a.e.set_label_object_group(duel, Some(d.into()));
        rock_earth(at, duel) && at.is_faceup(duel) && d.is_relate_to_battle(duel) && d.is_location(duel, LOCATION_ONFIELD)
    } else {
        false
    }
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.is_relate_to_battle(duel)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
