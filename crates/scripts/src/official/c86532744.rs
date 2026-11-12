// Number S39: Utopia Prime
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 86532744;
const NUMBER_39_UTOPIA: u32 = 84013237;
pub const XYZ_NUMBER: u32 = 39;

pub const LISTED_NAMES: &[u32] = &[NUMBER_39_UTOPIA];

// In the Extra Deck, Xyz Summoned with three Level 4 LIGHT monsters or over Number 39: Utopia. The effect needs the opponent to be
// 3000 LP ahead, which random play reaches in only 2 of 64 scenarios and never in the first 13.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2863439, MZONE),
    (10992251, MZONE),
    (12493482, MZONE),
    (NUMBER_39_UTOPIA, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1)))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.get_lp(1 - a.tp) >= duel.get_lp(a.tp) + 3000);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<3, 3>, Cost::pay_lp_until::<10>])
    });
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[NUMBER_39_UTOPIA]) && c.is_faceup(duel)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_special_summoned(duel) && c.is_able_to_remove(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 300);
    Ok(false)
}

fn rmctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_REMOVED) && !c.is_reason(duel, REASON_REDIRECT)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() && duel.destroy_full(g, REASON_EFFECT, LOCATION_REMOVED, duel.get_reason_player())? > 0 {
        let ct = duel.get_operated_group().filter_count(duel, rmctfilter, None);
        if ct > 0 {
            duel.break_effect()?;
            duel.damage(1 - tp, ct as u32 * 300, REASON_EFFECT)?;
        }
    }
    Ok(())
}
