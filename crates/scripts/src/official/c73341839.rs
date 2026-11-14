// Miniborrel Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 73341839;

pub const LISTED_SERIES: &[u16] = &[SET_BORREL, SET_ROKKET];
// In the Graveyard with Borreload Dragon (a "Borrel" Link Monster) on the field, and Linguriboh (Link 1) to release.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (31833038, MZONE), (24842059, MZONE), (24842059, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_ROKKET], Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BORREL]) && c.is_link_monster(duel)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn rfilter(c: CardId, duel: &Duel) -> bool {
    c.is_link_below(duel, 3)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(
            c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, tp, 0xff)
                && duel.check_release_group_cost(tp, rfilter, (1, 1), false, Some(&aux::release_check_mmz), None),
        );
    }
    let rg = duel.select_release_group_cost(tp, rfilter, (1, 1), false, Some(&aux::release_check_mmz), None)?;
    let lk = rg.get_first().map_or(0, |r| r.get_link(duel));
    e.set_label(duel, i64::from(lk));
    duel.release(rg, REASON_COST)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP_ATTACK)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
        e1.set_value(duel, Value::Card(lnklimit));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_label(duel, e.get_label(duel));
        c.register_effect_full(duel, e1, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn lnklimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(c.is_link(duel, &[e.get_label(duel) as u32]))
}
