// Black Feather Whirlwind
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, SZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 7602800;

pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_WINGED_DRAGON];

// Blackwing - Blizzard the Far North and Blackwing - Zephyros the Elite on the field to Synchro Summon Blackwing - Boreastorm the Wicked Wind,
// with Blackwing - Bora the Spear in the GY to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, SZONE),
    (10602628, EXTRA),
    (10602628, EXTRA),
    (22835145, MZONE),
    (22835145, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (49003716, GRAVE),
    (49003716, GRAVE),
];
// The destruction replacement needs a Feather Counter, which only Black-Winged Dragon taking effect damage places.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_SELECT_EFFECTYN)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, desreptg);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e3.set_operation(duel, desrepop);
    c.register_effect(duel, e3);
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_summon_player(duel, tp)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_summon_location(duel, LOCATION_EXTRA)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spcfilter(c, duel, tp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, atk: i64) -> bool {
    (c.is_set_card(duel, &[SET_BLACKWING]) || c.is_code(duel, CARD_BLACK_WINGED_DRAGON))
        && c.is_faceup(duel)
        && i64::from(c.get_attack(duel)) < atk
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default().filter(duel, |c, duel| spcfilter(c, duel, tp), None);
    let atk = eg
        .get_max_group(duel, |c, duel| i64::from(c.get_attack(duel)))
        .map_or(0, |(_, atk)| atk);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp, atk);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_can_remove_counter(tp, 1, 0, COUNTER_FEATHER, 1, REASON_EFFECT));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove_counter(a.tp, 1, 0, COUNTER_FEATHER, 1, REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}
