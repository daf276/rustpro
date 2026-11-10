// Blizzard
use crate::prelude::*;
use crate::testing::{OPPONENT, SZONE};

const ID: u32 = 51706604;

// Elfnotes: Welcome Home and Rhapsodia of Madness on the opponent's side, whose effects it negates.
pub const SUPPORT: &[(u32, u16)] = &[
    (64491754, SZONE | OPPONENT),
    (64491754, SZONE | OPPONENT),
    (24092792, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_negatable_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_ONFIELD) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_CHAIN_SOLVING);
        e1.set_condition(duel, discon);
        e1.set_operation(duel, disop);
        e1.set_label(duel, tc.get_original_code_rule(duel).into());
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD_EXC_GRAVE | RESET_PHASE | PHASE_END);
        e2.set_condition(duel, |duel, a| {
            let c = a.e.get_handler(duel);
            c.is_faceup(duel) && c.get_destination(duel) == LOCATION_GRAVE
        });
        e2.set_value(duel, LOCATION_HAND as i32);
        tc.register_effect_full(duel, e2, true);
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    re.is_spell_effect(duel)
        && loc & LOCATION_ONFIELD != 0
        && re.get_handler(duel).get_original_code_rule(duel) == a.e.get_label(duel) as u32
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.negate_effect(a.ev as u8);
    Ok(())
}
