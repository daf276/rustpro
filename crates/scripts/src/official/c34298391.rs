// Mind Scan
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 34298391;

pub const LISTED_SERIES: &[u16] = &[SET_TOON];
// Toon World (a "Toon" Spell) in the hand and a Toon monster on the field for the declaration, with copies Set to activate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (CARD_TOON_WORLD, HAND),
    (CARD_TOON_WORLD, HAND),
    (42386471, MZONE),
    (21296502, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_PUBLIC);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_TOON]));
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_target_range(duel, 0, LOCATION_HAND | LOCATION_ONFIELD);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, declcon);
    e2.set_target(duel, decltg);
    e2.set_operation(duel, declop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn declconfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_type(duel, TYPE_TOON) && c.is_monster(duel)) || (c.is_set_card(duel, &[SET_TOON]) && c.is_spell(duel))) && c.is_faceup(duel)
}

fn declcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(declconfilter, a.tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_main_card_type(duel))) == 2
}

fn decltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let mut announce_filter = vec![u64::from(TYPE_NORMAL), OPCODE_ISTYPE, OPCODE_NOT];
    for i in 1..duel.get_current_chain() {
        let code1: u32 = duel.get_chain_info(i, CHAININFO_TRIGGERING_CODE);
        let code2: u32 = duel.get_chain_info(i, CHAININFO_TRIGGERING_CODE2);
        for code in std::iter::once(code1).chain((code2 > 0).then_some(code2)) {
            announce_filter.extend([u64::from(code), OPCODE_ISCODE, OPCODE_NOT, OPCODE_AND]);
        }
    }
    let declared_code = duel.announce_card(tp, &announce_filter)?;
    duel.set_target_param(declared_code);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    Ok(false)
}

fn declop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let declared_code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
        c.set_hint(duel, CHINT_CARD, declared_code.into());
        let e0 = duel.create_effect(c);
        e0.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e0.set_code(duel, EVENT_TURN_END);
        e0.set_range(duel, LOCATION_SZONE);
        e0.set_count_limit(duel, 1, 0);
        e0.set_operation(duel, |duel, a| {
            a.e.get_handler(duel).set_hint(duel, CHINT_CARD, 0);
            a.e.reset(duel);
            Ok(())
        });
        e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EVENT_CHAIN_SOLVING);
        e1.set_range(duel, LOCATION_SZONE);
        e1.set_label(duel, declared_code.into());
        e1.set_condition(duel, |duel, a| {
            a.re.is_some_and(|re| re.get_handler(duel).get_original_code_rule(duel) == a.e.get_label(duel) as u32)
        });
        e1.set_operation(duel, |duel, a| {
            duel.negate_effect(a.ev as u8);
            Ok(())
        });
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
