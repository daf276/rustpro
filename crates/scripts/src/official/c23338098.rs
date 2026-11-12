// Assault Blackwing - Chidori the Rain Sprinkling
use super::c16051717::{add_type_effects, tncon};
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 23338098;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck (Synchro Summoned from Gale, a Level 3 Tuner, and Zephyros, Level 4) and on the field, with Synchro
// Blackwing monsters in the Graveyard, destroyed by the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (2009101, MZONE),
    (2009101, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (10602628, GRAVE),
    (69031175, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    c.enable_revive_limit(duel);
    add_type_effects(duel, c, tncon);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(
        duel,
        Value::Card(|duel, _, c| 300 * duel.get_matching_group_count(atkfilter, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32),
    );
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE));
    e4.set_target(duel, sptg);
    e4.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_BLACKWING])
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_SYNCHRO)
        && c.is_race(duel, RACE_WINGEDBEAST)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !c.is_code(duel, ID)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
