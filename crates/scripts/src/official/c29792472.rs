// Yad'al-Hecahands
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 29792472;

pub const LISTED_SERIES: &[u16] = &[SET_HECAHANDS];
// In the hand and Set, with Hecahands Gaigas on the field to negate the opponent's Set Mystical Space Typhoon and Pot of Greed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (95132593, MZONE),
    (5318639, SZONE | OPPONENT),
    (55144522, HAND | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_HECAHANDS]),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
        && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_relate_to_effect(duel, re) && rc.is_destructable(duel) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
            duel.set_possible_operation_info(0, CATEGORY_SET, Some(rc.into()), 1, tp, 0);
        }
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_activation(a.ev as u8)
        && rc.is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
        && !rc.is_location(duel, LOCATION_HAND | LOCATION_DECK)
        && aux::nvfilter(rc, duel)
        && rc.is_ssetable(duel)
        && rc.is_spell_trap(duel)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.sset(tp, rc)?;
    }
    Ok(())
}
