// Spright Red
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 75922381;

// Psychic Kappa, a Level 2 Normal Monster, to Special Summon it and to release.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (7892180, MZONE), (7892180, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_level(duel, &[2]) || c.is_link(duel, &[2]))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
}

fn discostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[2]) || c.is_rank(duel, &[2]) || c.is_link(duel, &[2])
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, discostfilter, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, discostfilter, (1, 1), false, None, Some(c.into()))?;
    let Some(rc) = g.get_first() else { return Ok(false) };
    e.set_label(duel, i64::from(rc.get_level(duel)));
    duel.release(rc, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    if chk == 0 {
        return Ok(!re.get_handler(duel).is_disabled(duel));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.negate_effect(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
        && e.get_label(duel) == 0
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        if let Some(eg) = a.eg {
            duel.destroy(eg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
