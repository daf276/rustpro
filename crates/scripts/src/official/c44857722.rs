// Full Moon Mirror
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 44857722;
const COUNTER_FULL_MOON: u16 = 0x219;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_FULL_MOON];

// Set, with Dark Hole and Raigeki for the counters and Kozaky (Fiend, Level 1) to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (53129443, HAND),
    (53129443, HAND),
    (12580477, HAND),
    (99171160, HAND),
    (99171160, MZONE | OPPONENT),
    (99171160, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_FULL_MOON);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_monster(duel) && !c.is_previous_location(duel, LOCATION_ONFIELD)) || c.is_previous_location(duel, LOCATION_MZONE))
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let des_count = a.eg.unwrap_or_default().filter_count(duel, ctfilter, None);
    if des_count == 0 {
        return Ok(());
    }
    if !duel.is_chain_solving() {
        c.add_counter(duel, COUNTER_FULL_MOON, des_count as u16);
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_operation(duel, |duel, a| {
        let e = a.e;
        e.get_handler(duel).add_counter(duel, COUNTER_FULL_MOON, e.get_label(duel) as u16);
        Ok(())
    });
    e1.set_label(duel, des_count as i64);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_operation(duel, |duel, a| {
        if let Some(e1) = a.e.get_label_object(duel) {
            e1.reset(duel);
        }
        Ok(())
    });
    e2.set_label_object(duel, Some(e1));
    e2.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e2, tp);
    Ok(())
}

fn lv6spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 6) && c.is_race(duel, RACE_FIEND | RACE_FAIRY) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[10]) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_able_to_hand(duel)
}

fn lv10spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[10]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let b1 = c.is_can_remove_counter(duel, tp, COUNTER_FULL_MOON, 1, REASON_COST)
        && mmz_chk
        && duel.is_existing_matching_card(
            |c, duel| lv6spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            None,
        );
    let b2 = c.is_can_remove_counter(duel, tp, COUNTER_FULL_MOON, 3, REASON_COST)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b3 = c.is_can_remove_counter(duel, tp, COUNTER_FULL_MOON, 5, REASON_COST)
        && mmz_chk
        && duel.is_existing_matching_card(
            |c, duel| lv10spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            None,
        );
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let Some(op) = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?
    else {
        return Ok(false);
    };
    e.set_label(duel, op as i64);
    c.remove_counter(duel, tp, COUNTER_FULL_MOON, 2 * op as u16 - 1, REASON_COST)?;
    if op == 1 || op == 3 {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    } else if op == 2 {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if (op == 1 || op == 3) && mmz_chk {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = |c: CardId, duel: &Duel| {
            if op == 1 {
                lv6spfilter(c, duel, e, tp)
            } else {
                lv10spfilter(c, duel, e, tp)
            }
        };
        let g = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(filter),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
