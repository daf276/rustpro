// Princess Dracaena, the Thirsting Tree
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 79966218;

pub const LISTED_NAMES: &[u32] = &[ID];

// In hand and in the Graveyard, with Green Phantom King in the Deck and the Graveyard and Petit Dragon in hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (22910685, DECK),
    (22910685, DECK),
    (22910685, GRAVE),
    (75356564, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[selfspcost, effcost]));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[dragonspcost, effcost]));
    e2.set_target(duel, dragonsptg);
    e2.set_operation(duel, dragonspop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_race(duel, RACE_DRAGON | RACE_PLANT)),
    );
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_race_except(duel, RACE_DRAGON | RACE_PLANT));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_PLANT) && !c.is_code(duel, ID) && c.is_able_to_grave_as_cost(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(selfspcostfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    if let Some(sc) = duel
        .select_matching_card(tp, selfspcostfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    {
        duel.send_to_grave(sc, REASON_COST)?;
        let level = sc.get_level(duel);
        a.e.set_label(duel, i64::from(level));
    }
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, e.get_label(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn dragonspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn dragonspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, handler: CardId) -> bool {
    c.is_race(duel, RACE_PLANT)
        && c.is_able_to_remove_as_cost(duel)
        && duel.is_existing_matching_card(
            |c, duel| dragonspfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            Some(CardSet::from_iter([c, handler])),
        )
}

fn dragonspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |c2, duel: &Duel| dragonspcostfilter(c2, duel, e, tp, c);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn dragonsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| dragonspfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_GRAVE,
    )
}

fn dragonspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(move |c, duel| dragonspfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
