// Six Strike - Triple Impact
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 81426505;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
// In the hand beside three Six Samurai monsters (Legendary Six Samurai - Kageki), with monsters and Spells and Traps of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (2511717, MZONE),
    (2511717, MZONE),
    (2511717, MZONE),
    (1184620, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_SIX_SAMURAI])),
        a.tp,
        LOCATION_MZONE,
        0,
        3,
        None,
    )
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel)
}

fn destroy_group(duel: &Duel, tp: u8, opt: u64) -> CardSet {
    match opt {
        0 => duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None),
        1 => duel.get_matching_group(filter2, tp, 0, LOCATION_ONFIELD, None),
        _ => duel.get_matching_group(CardId::is_facedown, tp, 0, LOCATION_SZONE, None),
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let available = [
        duel.is_existing_matching_card(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None),
        duel.is_existing_matching_card(filter2, tp, 0, LOCATION_ONFIELD, 1, None),
        duel.is_existing_matching_card(CardId::is_facedown, tp, 0, LOCATION_SZONE, 1, None),
    ];
    if chk == 0 {
        return Ok(available.contains(&true));
    }
    let t: Vec<u64> = (0..3).filter(|&n| available.get(n as usize) == Some(&true)).collect();
    let descriptions: Vec<u64> = t.iter().map(|&n| aux::stringid(ID, n)).collect();
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let sel = duel.select_option(tp, &descriptions)?;
    let opt = usize::try_from(sel).ok().and_then(|sel| t.get(sel)).copied().unwrap_or(0);
    let sg = destroy_group(duel, tp, opt);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    e.set_label(duel, opt as i64);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let sg = destroy_group(duel, a.tp, a.e.get_label(duel) as u64);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
