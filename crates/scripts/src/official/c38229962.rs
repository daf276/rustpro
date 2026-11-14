// Giant Beetrooper Invincible Atlas
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 38229962;

pub const LISTED_SERIES: &[u16] = &[SET_BEETROOPER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_INSECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Insects (Petit Moth), releasing an Insect to gain ATK or to Special Summon Beetrooper Scout Buggy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2656842, DECK),
    (2656842, DECK),
    (58192742, MZONE),
    (58192742, MZONE),
    (ID, EXTRA),
    (58192742, MZONE),
    (58192742, MZONE),
    (58192742, MZONE),
    (58192742, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(2656842),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, protcon);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_INSECT));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, effcost);
    e4.set_target(duel, efftg);
    e4.set_operation(duel, effop);
    c.register_effect(duel, e4);
}

fn protcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_link_summoned(duel) && c.is_attack_below(duel, 3000)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BEETROOPER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn costfilter(c: CardId, duel: &Duel, tp: u8, tc: CardId, spcheck: bool) -> bool {
    c.is_race(duel, RACE_INSECT) && (c != tc || (spcheck && duel.get_mzone_count(tp, Some(c.into())) > 0))
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let spcheck = duel.is_existing_matching_card(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |rc, duel| costfilter(rc, duel, tp, c, spcheck), (1, 1), false, None, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_release_group_cost(tp, |rc, duel| costfilter(rc, duel, tp, c, spcheck), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None);
    let b2 = e.get_handler(duel).is_location(duel, LOCATION_MZONE);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_ATKCHANGE);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == 2 {
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 2000);
            e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
