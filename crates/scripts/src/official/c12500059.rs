// Dawn Angel Kambi
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 12500059;

// Copies in the hand and on the field with Fairies (Binding Chain, Herald of Orange Light, Doma, Light Bringer Lucifer) to Tribute and
// to Special Summon from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (8058240, MZONE),
    (8058240, HAND),
    (8058240, DECK),
    (17266660, DECK),
    (16972957, DECK),
    (21251800, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(selfspconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, deckspcost);
    e2.set_target(duel, decksptg);
    e2.set_operation(duel, deckspop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(|duel, c| c.is_race(duel, RACE_FAIRY)));
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_race(duel, RACE_FAIRY)
}

fn deckspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FAIRY) && c.has_level(duel)
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: i64) -> bool {
    c.is_race(duel, RACE_FAIRY) && c.is_level(duel, &[lv as u32]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn deckspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    let c = e.get_handler(duel);
    let rescon = |duel: &Duel, sg: CardSet, tp: u8, _: CardSet| {
        let res = sg.contains(c)
            && duel.get_mzone_count(tp, Some(sg)) > 0
            && duel.is_existing_matching_card(
                |dc, duel| deckspfilter(dc, duel, e, tp, sg.get_sum(duel, |sc, duel| i64::from(sc.get_level(duel)))),
                tp,
                LOCATION_DECK,
                0,
                1,
                None,
            );
        (res, !sg.contains(c))
    };
    if chk == 0 {
        return Ok(c.is_releasable(duel)
            && c.has_level(duel)
            && duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0
            && duel.check_release_group_cost(tp, deckspcostfilter, (1, 99), false, Some(&rescon), None));
    }
    let mut g = duel.select_release_group_cost(tp, deckspcostfilter, (1, 99), false, Some(&rescon), None)?;
    g.insert(c);
    let level = g.get_sum(duel, |sc, duel| i64::from(sc.get_level(duel)));
    e.set_label(duel, level);
    duel.release(g, REASON_COST)?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_FAIRY));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn decksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let res = a.e.get_label(duel) == -100;
        a.e.set_label(duel, 0);
        return Ok(res);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn deckspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = e.get_label(duel);
    let g = duel.select_matching_card(tp, |c, duel| deckspfilter(c, duel, e, tp, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
