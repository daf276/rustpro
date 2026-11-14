// Virtual World Tiger - Fufu
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 95243515;

pub const LISTED_SERIES: &[u16] = &[SET_VIRTUAL_WORLD, SET_VIRTUAL_WORLD_GATE];
// On the field with Xyz Import to attach Sangan, against the opponent's Sangan of another Type and Attribute. No Virtual World Gate
// is ported, so its direct attack and immunity are not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    e2.set_condition(duel, effcond::<2>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, effcond::<4>);
    e3.set_value(
        duel,
        Value::Effect(|duel, _, te| te.is_activated(duel) && !te.is_card_setcode(duel, SET_VIRTUAL_WORLD)),
    );
    c.register_effect(duel, e3);
}

fn effcond<const VALUE: usize>(duel: &Duel, a: &Args) -> bool {
    let ct = duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_VIRTUAL_WORLD_GATE])),
        a.e.get_handler_player(duel),
        LOCATION_ONFIELD,
        0,
        None,
    );
    (VALUE == 2 && ct >= VALUE) || (VALUE == 4 && ct == VALUE)
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel) && c.is_can_be_effect_target(duel, e)
}

fn propfilter(c: CardId, duel: &Duel, sc: Option<CardId>) -> bool {
    sc.is_some_and(|sc| !c.is_race(duel, sc.get_race(duel)) && !c.is_attribute(duel, sc.get_attribute(duel)))
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let first = sg.get_first();
    (
        sg.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None)
            && sg.is_exists(duel, |c, duel| propfilter(c, duel, first), 1, None),
        false,
    )
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tgfilter(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| {
            rescon(duel, sg, tp)
        }));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp), tp, HINTMSG_NEGATE)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(tg), 2, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let dg = duel.get_target_cards(e).unwrap_or_default();
    if dg.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for tc in dg {
        if tc.is_faceup(duel) && !tc.is_disabled(duel) {
            duel.negate_related_chain(tc, RESET_TURN_SET);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_value(duel, RESET_TURN_SET as i32);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
