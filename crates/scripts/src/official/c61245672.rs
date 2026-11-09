// Decode Talker Heatsoul
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 61245672;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(99);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_attribute(duel))
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// Link Summoned from Cyberse monsters of different Attributes (Leotron, Firewall Dragon) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (47226949, MZONE),
    (47226949, MZONE),
    (5043010, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, c| c.get_linked_group(duel).filter_count(duel, CardId::is_monster, None) as i32 * 500),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::pay_lp::<1000>);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_link_below(duel, 3)
        && c.is_race(duel, RACE_CYBERSE)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? > 0
        && duel.get_lp(tp) <= 2000
        && c.is_relate_to_effect(duel, e)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|g, duel| spfilter(g, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        if duel.remove(c, POS_FACEUP, REASON_EFFECT)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |g, duel| spfilter(g, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}
