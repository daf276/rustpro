// Gouki The Powerload Ogre
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 11516241;

pub const LISTED_SERIES: &[u16] = &[SET_GOUKI];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_WARRIOR, Some(lc), sumtype, tp)
    }
}

// Link Summoned from Gouki Guts and Gouki Machine Suprex (Warriors), with Gouki The Solid Ogre to release and cards to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (22510667, MZONE),
    (7540107, MZONE | OPPONENT),
    (29099860, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let g = duel.get_matching_group(
        |c, duel| c.is_type(duel, TYPE_LINK),
        c.get_controler(duel),
        LOCATION_MZONE,
        0,
        Some(c.into()),
    );
    g.get_sum(duel, |c, duel| i64::from(c.get_link(duel))) as i32 * 200
}

fn descost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 100);
    Ok(true)
}

fn spcheck(duel: &Duel, sg: CardSet, dg: CardSet) -> bool {
    let a = sg
        .iter()
        .map(|c| i32::from(dg.is_contains(c)) + c.get_equip_group(duel).iter().filter(|&tc| dg.is_contains(tc)).count() as i32)
        .sum::<i32>();
    dg.len() as i32 - a > 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOUKI]) && c.is_type(duel, TYPE_LINK)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let dg = duel.get_matching_group(
        |c, duel| c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    let check = |duel: &Duel, sg: CardSet, _: u8, _: CardSet| (spcheck(duel, sg, dg), false);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.check_release_group_cost(tp, cfilter, (1, 1), false, Some(&check), None));
    }
    let sg = duel.select_release_group_cost(tp, cfilter, (1, 1), false, Some(&check), None)?;
    let lnk = sg.get_first().map_or(0, |c| c.get_link(duel));
    duel.release(sg, REASON_COST)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, lnk as u8, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, dg, dg.map_or(0, |dg| dg.len() as u8), tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let dg = duel.get_target_cards(a.e).unwrap_or_default();
    if !dg.is_empty() {
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
