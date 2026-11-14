// Machine Angel Ritual
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 39996157;

pub const LISTED_SERIES: &[u16] = &[SET_CYBER_ANGEL];

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_CYBER_ANGEL])
    }
}

// In the hand to Ritual Summon Cyber Angel Benten with two Mystical Elf on the field, and in the Graveyard with a LIGHT monster on
// the field destroyed by the opponent's Raigeki. The replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (77235086, HAND),
    (15025844, MZONE),
    (15025844, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(77235086), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater::<Params>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, repop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT | REASON_BATTLE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && a.eg
                .is_some_and(|eg| eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None)));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
