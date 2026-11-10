// Ib'al-Hecahands
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 93294363;

// In the hand and Set, with Hecahands Gaigas on the field to negate the monster effects of the opponent's Sangan and Mulcharmy
// Fuwalos.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (95132593, MZONE),
    (26202165, MZONE | OPPONENT),
    (42141493, HAND | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel))
        && a.rp == 1 - a.tp
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
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(rc.into()), 1, tp, 0);
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
        && rc.is_monster(duel)
        && rc.is_can_be_special_summoned(duel, a.e, 0, tp, false, false)
    {
        let blocked = if rc.is_location(duel, LOCATION_EXTRA) {
            duel.get_location_count_from_ex(tp, tp, None, rc) <= 0
        } else {
            duel.get_location_count(tp, LOCATION_MZONE) <= 0
        };
        if !blocked && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            duel.special_summon(rc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
