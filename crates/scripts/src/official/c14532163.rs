// Lightning Storm
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 14532163;
const CHOICE: u8 = 0;

// Sangan and Mystical Space Typhoon of the opponent, for each of the two destroy choices.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_ONFIELD, 0, 1, None)
    });
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn targets(duel: &Duel, tp: u8, choice: usize) -> CardSet {
    if choice == 1 {
        duel.get_matching_group(CardId::is_attack_pos, tp, 0, LOCATION_MZONE, None)
    } else {
        duel.get_matching_group(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None)
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (b1, b2) = (!targets(duel, tp, 1).is_empty(), !targets(duel, tp, 2).is_empty());
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 0)), (b2, aux::stringid(ID, 1))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    let g = targets(duel, tp, choice.unwrap_or(0));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let choice = a.e.get_chain_data(duel, CHOICE).unwrap_or(0);
    if choice == 1 || choice == 2 {
        let g = targets(duel, a.tp, choice as usize);
        if !g.is_empty() {
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
