// Rose Girl
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 29177818;

// In hand and in the Graveyard, with Mushroom Man (Level 2 Plant Normal Monster) on the field for Dark Hole to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (14181608, MZONE), (14181608, MZONE), (53129443, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn confilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_previous_race_on_field(duel) & RACE_PLANT == RACE_PLANT
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| confilter(c, duel, tp), 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_PLANT)),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        None,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    }
    Ok(())
}
