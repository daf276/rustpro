// R.B. Ga10 Cutter
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 33438265;

pub const LISTED_SERIES: &[u16] = &[SET_RB];
// R.B. Link Monsters to link with, and the opponent's set Spells and Traps to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (32216688, MZONE),
    (6821579, MZONE),
    (32216688, EXTRA),
    (75652080, SZONE | OPPONENT),
    (75652080, SZONE | OPPONENT),
    (75652080, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_cost(duel, Cost::pay_lp::<700>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

pub fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, None);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && (g.is_empty() || g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_RB]), None) == g.len())
}

fn desconfilter(c: CardId, duel: &Duel, ec: CardId, lg: CardSet) -> bool {
    c.is_set_card(duel, &[SET_RB])
        && c.is_link_monster(duel)
        && c.is_faceup(duel)
        && (lg.contains(c) || c.get_linked_group(duel).contains(ec))
}

pub fn is_linked_with_rb(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let lg = c.get_linked_group(duel);
    duel.is_existing_matching_card(
        |tc, duel| desconfilter(tc, duel, c, lg),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        Some(c.into()),
    )
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
        && duel.is_chain_disablable(a.ev as u8)
        && is_linked_with_rb(duel, a)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    if a.re.is_some_and(|re| {
        let rc = re.get_handler(duel);
        rc.is_relate_to_effect(duel, re) && rc.is_destructable(duel)
    }) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(eg.union(c.into())), 2, tp, 0);
    } else {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(eg), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e)
        && duel.destroy(c, REASON_EFFECT)? > 0
        && duel.negate_effect(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
