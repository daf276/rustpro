// Odd-Eyes Revolution Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, PZONE};

const ID: u32 = 16306932;

// In the hand (to Special Summon by releasing a Fusion, Synchro and Xyz Dragon, or to discard for Supreme King Dragon Darkwurm),
// in the Pendulum Zone to revive a Dragon from the Graveyard, and on the field to shuffle everything.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (3366982, MZONE),
    (12496261, MZONE),
    (16195942, MZONE),
    (3366982, GRAVE),
    (69610326, DECK),
    (69610326, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(0),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    c.enable_unsummonable(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_REVIVE_LIMIT);
    e0.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_location(duel, LOCATION_HAND));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        !c.is_race(duel, RACE_DRAGON) && sumtype & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e3.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_SPSUMMON_PROC);
    e4.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e4.set_range(duel, LOCATION_HAND);
    e4.set_condition(duel, hspcon);
    e4.set_target(duel, hsptg);
    e4.set_operation(duel, hspop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_HAND);
    e5.set_cost(duel, thcost);
    e5.set_target(duel, thtg);
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_code(duel, EFFECT_UPDATE_ATTACK);
    e6.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_value(duel, Value::Card(|duel, e, _| duel.get_lp(1 - e.get_handler_player(duel)) / 2));
    c.register_effect(duel, e6);
    let e7 = e6.clone_effect(duel);
    e7.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e7);
    let e8 = duel.create_effect(c);
    e8.set_category(duel, CATEGORY_TODECK);
    e8.set_type(duel, EFFECT_TYPE_IGNITION);
    e8.set_range(duel, LOCATION_MZONE);
    e8.set_count_limit(duel, 1, 0);
    e8.set_cost(duel, tdcost);
    e8.set_target(duel, tdtg);
    e8.set_operation(duel, tdop);
    c.register_effect(duel, e8);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON)
        && c.is_type(duel, TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel
        .select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(e.get_handler(duel).into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && duel.destroy(c, REASON_EFFECT)? != 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e))
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    st & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM && e.get_handler(duel).is_location(duel, LOCATION_HAND)
}

fn release_groups(duel: &Duel, tp: u8) -> [CardSet; 3] {
    let rg = duel.get_release_group(tp).filter(
        duel,
        |c, duel| c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ),
        None,
    );
    [TYPE_FUSION, TYPE_SYNCHRO, TYPE_XYZ].map(|t| rg.filter(duel, |c, duel| c.is_type(duel, t), None))
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let groups = release_groups(duel, tp);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && groups
            .iter()
            .all(|&g| !g.is_empty() && aux::select_unselect_group_check(duel, g, 1, 1, &aux::chkf_mmz(tp, 1)))
}

fn hsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let [g1, g2, g3] = release_groups(duel, tp);
    let select =
        |duel: &mut Duel, g| aux::select_unselect_group_full(duel, g, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_RELEASE, None, true);
    let mut mg1 = select(duel, g1)?;
    if !mg1.is_empty() {
        mg1.merge(select(duel, g2)?);
        if mg1.len() > 1 {
            mg1.merge(select(duel, g3)?);
        }
    }
    if mg1.len() == 3 {
        e.set_label_object_group(duel, Some(mg1));
        return Ok(true);
    }
    Ok(false)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST | REASON_MATERIAL)?;
    Ok(())
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_discardable(duel) && duel.check_lp_cost(a.tp, 500));
    }
    duel.send_to_grave(c, REASON_COST | REASON_DISCARD)?;
    duel.pay_lp_cost(a.tp, 500)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && c.is_race(duel, RACE_DRAGON) && c.is_level_below(duel, 8) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let paid = (duel.get_lp(a.tp) / 2) as u32;
    duel.pay_lp_cost(a.tp, paid)?;
    Ok(false)
}

fn tdgroup(duel: &Duel, a: &Args) -> CardSet {
    let c = a.e.get_handler(duel);
    let zones = LOCATION_ONFIELD | LOCATION_GRAVE;
    duel.get_matching_group(CardId::is_able_to_deck, a.tp, zones, zones, Some(c.into()))
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = tdgroup(duel, a);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = tdgroup(duel, a);
    if !g.is_empty() {
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
