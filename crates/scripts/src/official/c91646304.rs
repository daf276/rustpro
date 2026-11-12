// Crusadia Arboria
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 91646304;

// In the hand, to Special Summon to the zone Mekk-Knight Crusadia Avramax points to, on the field and in the Graveyard, with Dark
// Hole in the opponent's hand to destroy the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (ID, GRAVE), (21887175, MZONE), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target_range(duel, POS_FACEUP_DEFENSE, 0);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_value(
        duel,
        Value::SpSummonProcedure(|duel, _, c| (0, aux::get_mm_zones_pointed_to(duel, c.get_controler(duel)) as i32)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE | LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
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
        && c.is_set_card(duel, &[SET_CRUSADIA])
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT | REASON_BATTLE)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_status(duel, STATUS_DESTROY_CONFIRMED)
            && c.is_able_to_remove(duel)
            && a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.remove(a.e.get_handler(duel), POS_FACEUP, REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}
