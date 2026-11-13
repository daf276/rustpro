// Wind Unicorn Parallel, the Dracoslayer
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, PZONE};

const ID: u32 = 39016067;

// Two copies in the Pendulum Zones for the first effect, one on the field, and cards on both sides to return to the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_PZONE, 0, 1, Some(a.e.get_handler(duel).into()))
    });
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE);
    e2a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2a.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_pendulum_summoned(duel));
    e2a.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2b.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, rthtg);
    e3.set_operation(duel, rthop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 5) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mmz_chk = duel.get_mzone_count(tp, None) > 0;
    let b1 = mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let b2 = mmz_chk && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_PZONE, 0, 1, Some(c.into()));
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_PZONE as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    let c = e.get_handler(duel);
    if op == 1 {
        if c.is_relate_to_effect(duel, e) {
            duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == 2 && c.is_relate_to_effect(duel, e) && duel.destroy(c, REASON_EFFECT)? > 0 && duel.get_mzone_count(tp, None) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_PZONE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn rthfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    ((c.is_original_type(duel, TYPE_PENDULUM) && c.is_faceup(duel)) || c.is_controler(duel, opp)) && c.is_able_to_hand(duel)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_target_group(|c, duel| rthfilter(c, duel, 1 - tp), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let rescon = aux::dpcheck(|c, duel| i64::from(c.get_controler(duel)));
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_RTOHAND)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), 2, tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
