// Magnet Warrior Sigma Plus
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 51826619;

pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR];
pub const LISTED_NAMES: &[u32] = &[ID];

// Two on the field with the Normal Magnet Warriors besides them and in the GY to return, and the opponent's Dark Hole to send
// them there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (99785935, MZONE),
    (39256679, GRAVE),
    (11549357, GRAVE),
    (99785935, GRAVE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_MUST_ATTACK);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_target_range(duel, 0, LOCATION_MZONE);
    e1a.set_condition(duel, |duel, a| earth_exists(duel, a, LOCATION_MZONE, 0));
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e1b.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_attribute(duel, ATTRIBUTE_EARTH))));
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_PATRICIAN_OF_DARKNESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, |duel, a| earth_exists(duel, a, 0, LOCATION_MZONE));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, thsptg);
    e3.set_operation(duel, thspop);
    c.register_effect(duel, e3);
}

fn earth_exists(duel: &Duel, a: &Args, location1: u32, location2: u32) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_attribute(duel, ATTRIBUTE_EARTH));
    duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), location1, location2, 1, None)
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_level_below(duel, 4)
        && c.is_set_card(duel, &[SET_MAGNET_WARRIOR])
        && !c.is_code(duel, ID)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && thspfilter(chkc, duel, e, tp, mmz_chk));
    }
    let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, mmz_chk);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |_, duel| duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 2),
        )?;
    }
    Ok(())
}
