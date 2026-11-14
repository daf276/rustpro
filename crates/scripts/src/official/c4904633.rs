// Shaddoll Core
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 4904633;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];
pub const LISTED_NAMES: &[u32] = &[ID];

// Set, to be activated as a monster, with Dark Hole to destroy it and a Shaddoll Spell in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (44394295, GRAVE), (53129443, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPE_MONSTER | TYPE_EFFECT, 1450, 1950, 9, RACE_SPELLCASTER, ATTRIBUTE_DARK);
    duel.is_player_can_special_summon_monster(tp, ID, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon(duel, tp) {
        return Ok(());
    }
    c.add_monster_attribute(duel, TYPE_EFFECT + TYPE_TRAP);
    duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
    c.add_monster_attribute_complete(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, ID);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect_full(duel, e1, true);
    duel.special_summon_complete()?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
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
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
