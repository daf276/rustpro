// Number 41: Bagooska the Terribly Tired Tapir
use engine::constants::MSG_CHAIN_DISABLED;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 90590303;
pub const XYZ_NUMBER: u32 = 41;

// In the Extra Deck, Xyz Summoned with Mystical Elf, Flying Kamakiri #2 and Maiden of the Moonlight, and the opponent's
// monsters with effects to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
// None of its effects is a chain link, so Resolves cannot hold.
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID), Check::Report(&Check::Message(MSG_CHAIN_DISABLED))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, mtop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_attack_pos(duel));
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(|duel, e, _, tp| tp != e.get_handler_player(duel)));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_SET_POSITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    e4.set_target_card(duel, |duel, _, c| c.is_faceup(duel));
    e4.set_value(duel, POS_FACEUP_DEFENSE as i32);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_code(duel, EVENT_CHAIN_SOLVING);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_condition(duel, discon);
    e5.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e5);
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.check_remove_overlay_card(duel, a.tp, 1, REASON_COST) {
        c.remove_overlay_card(duel, a.tp, 1, 1, REASON_COST)?;
    } else {
        duel.destroy(c, REASON_COST)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let pos: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_POSITION);
    a.e.get_handler(duel).is_defense_pos(duel)
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && loc == LOCATION_MZONE
        && pos & POS_DEFENSE != 0
}
