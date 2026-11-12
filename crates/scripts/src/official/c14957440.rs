// Otoshidamashi
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 14957440;
const COUNTER_OTOSHIDAMASHI: u16 = 0x59;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// On the field, with Raigeki to send the opponent's Sangans to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12580477, HAND),
    (12580477, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_OTOSHIDAMASHI);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atklm);
    e1.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_COUNTER + CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, tkcon);
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_BEAST, ATTRIBUTE_EARTH)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || (c.is_faceup(duel) && !c.is_type(duel, TYPE_TUNER))
}

fn atklm(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(atkfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && c.is_monster(duel) && c.is_previous_location(duel, LOCATION_MZONE)
}

fn tkcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_can_add_counter(duel, COUNTER_OTOSHIDAMASHI, 1));
    }
    duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, 0, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if c.add_counter(duel, COUNTER_OTOSHIDAMASHI, 1) && ft > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()) {
        duel.break_effect()?;
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let mut ct = i32::from(c.get_counter(duel, COUNTER_OTOSHIDAMASHI));
        if ct > ft {
            ct = ft;
        }
        if ct > 1 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_NUMBER);
            ct = duel.announce_number_range(tp, 1, ct as u32)? as i32;
        }
        for _ in 0..ct {
            let Some(token) = duel.create_token(tp, ID + 1) else { continue };
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
            let counters = i32::from(c.get_counter(duel, COUNTER_OTOSHIDAMASHI));
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_LEVEL);
            e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
            e1.set_value(duel, counters);
            token.register_effect_full(duel, e1, true);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_SET_ATTACK);
            e2.set_value(duel, (counters + 1) * 500);
            e2.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
            token.register_effect_full(duel, e2, true);
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_code(duel, EFFECT_SET_DEFENSE);
            e3.set_value(duel, (counters + 1) * 500);
            e3.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
            token.register_effect_full(duel, e3, true);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
