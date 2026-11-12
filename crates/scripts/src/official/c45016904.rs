// Tao Tao the Chanter
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 45016904;

// Copies on the field, with Level 4 Illusions (Jongleur-Ghoul Illusionist, Poplar of the White Forest) to Special Summon, and strong attackers
// of the opponent (Pesanta the Titan of Troubadours, Elfnote Lucina) to deal the damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (47894537, HAND),
    (47894537, GRAVE),
    (47894537, DECK),
    (47894537, DECK),
    (5800323, EXTRA),
    (5800323, EXTRA),
    (66429798, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, indestg);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.ep == a.tp && a.r & (REASON_BATTLE | REASON_EFFECT) > 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn indestg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let handler = e.get_handler(duel);
    c == handler || Some(c) == handler.get_battle_target(duel)
}

// The Lua passes the Monster Zone count as mmz_chk, a number, which is truthy even at 0.
fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_level_above(duel, 3) && c.is_race(duel, RACE_ILLUSION) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    let is_in_extra_deck = c.is_location(duel, LOCATION_EXTRA);
    !is_in_extra_deck || duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

fn locations(ev: u32) -> u32 {
    LOCATION_HAND | LOCATION_GRAVE | if ev >= 2000 { LOCATION_DECK | LOCATION_EXTRA } else { 0 }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = locations(a.ev);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, locations(a.ev), 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
