// Drytron Fafnir
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 58793369;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
pub const LISTED_NAMES: &[u32] = &[ID];
// Drytron Spells to add, a face-up Drytron monster, and Drytron monsters to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (94187078, DECK),
    (21576077, DECK),
    (94187078, HAND),
    (1174075, MZONE),
    (97148796, HAND),
    (97148796, DECK),
    (96026108, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    for code in [EFFECT_CANNOT_INACTIVATE, EFFECT_CANNOT_DISEFFECT] {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_FIELD);
        e.set_code(duel, code);
        e.set_range(duel, LOCATION_FZONE);
        e.set_value(duel, Value::Chain(effectfilter));
        c.register_effect(duel, e);
    }
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e4.set_condition(duel, lvcond);
    e4.set_target(duel, lvtg);
    e4.set_operation(duel, lvop);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_SUMMON_SUCCESS);
    c.register_effect(duel, e5);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_set_card(duel, &[SET_DRYTRON]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn effectfilter(duel: &Duel, _: EffectId, ct: u8) -> bool {
    duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT)
        .is_some_and(|te| te.get_handler(duel).is_ritual_spell(duel))
}

fn lvcond(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DRYTRON]));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && c.is_level_above(duel, 2) && c.is_attack_above(duel, 1000)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.eg.is_some_and(|eg| eg.is_exists(duel, lvfilter, 1, None)));
    }
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(eg) = a.eg else { return Ok(()) };
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    for tc in eg.filter(duel, lvfilter, None) {
        let lv = tc.get_attack(duel) / 1000;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -lv);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
