// Tenpai Dragon Paidra
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 39931513;

// Tenpai Dragon Chundra, a Level 4 Tuner, on the field to Synchro Summon with Ancient Fairy Dragon in the Battle Phase. No
// Sangen Spell/Trap is ported, so the search and the no battle damage are not reached.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (91810826, MZONE), (25862681, EXTRA), (25862681, EXTRA)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| {
        c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_DRAGON)
    });
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, _| duel.is_battle_phase());
    e4.set_target(duel, synchtg);
    e4.set_operation(duel, synchop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_set_card(duel, &[SET_SANGEN]) && (c.is_able_to_hand(duel) || c.is_ssetable(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            CardId::is_ssetable,
            |duel| duel.sset(tp, tc).map(drop),
            aux::stringid(ID, 3),
        )?;
    }
    Ok(())
}

pub fn synchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(
            |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

pub fn synchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_controler(duel, 1 - tp) || !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), None)?;
        }
    }
    Ok(())
}
