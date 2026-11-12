// Shipping Archfiend
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 57256127;

// On the field with Elfnote Lucinas on both sides (same Type and Attribute), and the opponent's Monster Reborn to Special Summon another.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13597785, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, GRAVE | OPPONENT),
    (83764718, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, changetg);
    e1.set_operation(duel, changeop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| !eg.contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - a.tp), 1, None))
    });
    e2.set_target(duel, rthtg);
    e2.set_operation(duel, rthop);
    c.register_effect(duel, e2);
}

fn single_property_filter(c: CardId, duel: &Duel, func: fn(CardId, &Duel) -> u64) -> bool {
    let property = func(c, duel);
    property & property.wrapping_sub(1) == 0
}

fn get_excluded_values(duel: &Duel, tg: CardSet, func: fn(CardId, &Duel) -> u64) -> u64 {
    tg.filter(duel, |c, duel| single_property_filter(c, duel, func), None)
        .get_bitwise_or(duel, func)
}

fn changetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        if !(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel)) {
            return Ok(false);
        }
        let &[op, val, ..] = e.get_labels(duel) else { return Ok(false) };
        return Ok((op == 1 && chkc.is_race_except(duel, val as u64)) || (op == 2 && chkc.is_attribute_except(duel, val as u32)));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let max_target_count = duel.get_field_group_count(tp, LOCATION_MZONE, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tg = duel
        .select_target(
            tp,
            CardId::is_faceup,
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            max_target_count as u8,
            None,
        )?
        .unwrap_or_default();
    let excluded_races = get_excluded_values(duel, tg, CardId::get_race);
    let excluded_attributes = get_excluded_values(duel, tg, |c, duel| u64::from(c.get_attribute(duel)));
    let b1 = excluded_races & RACE_ALL != RACE_ALL;
    let b2 = excluded_attributes & u64::from(ATTRIBUTE_ALL) != u64::from(ATTRIBUTE_ALL);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    unported!(duel, op.is_none());
    let decl_race_attr = match op {
        Some(1) => duel.announce_race(tp, 1, RACE_ALL & !excluded_races)?,
        Some(2) => u64::from(duel.announce_attribute(tp, 1, ATTRIBUTE_ALL & !(excluded_attributes as u32))?),
        _ => 0,
    };
    e.set_labels(duel, &[op.map_or(0, |op| op as i64), decl_race_attr as i64]);
    Ok(false)
}

fn changeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tg = duel.get_target_cards(e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if tg.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let &[op, decl_race_attr, ..] = e.get_labels(duel) else {
        return Ok(());
    };
    let eff_code = if op == 1 { EFFECT_CHANGE_RACE } else { EFFECT_CHANGE_ATTRIBUTE };
    for tc in tg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, eff_code);
        e1.set_value(duel, decl_race_attr as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.len() == 2
            && sg.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) == 2
            && sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == 1
            && sg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == 1,
        false,
    )
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_target_group(
        aux::faceup_filter(CardId::is_able_to_hand),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg)));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_RTOHAND)?;
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
