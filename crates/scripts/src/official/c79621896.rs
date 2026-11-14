// Archfiend's Origin
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 79621896;

pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];
pub const LISTED_NAMES: &[u32] = &[CARD_SUMMONED_SKULL];

// In hand and on the field with Summoned Skull on the field and in the Deck, and Archfiend Soldier to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_SUMMONED_SKULL, MZONE),
    (CARD_SUMMONED_SKULL, DECK),
    (CARD_SUMMONED_SKULL, DECK),
    (49881766, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, selfspcost);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_target(duel, lv6fiendsptg);
    e2a.set_operation(duel, lv6fiendspop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, negcon);
    e3.set_target(duel, stock::negate_destroy_target);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let archfiend = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_ARCHFIEND]);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, archfiend, (1, 1), true, Some(&aux::release_check_mmz), Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, archfiend, (1, 1), true, Some(&aux::release_check_mmz), Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn lv6fiendspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[6])
        && c.is_race(duel, RACE_FIEND)
        && c.is_attack(duel, &[2500])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn lv6fiendsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| lv6fiendspfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn lv6fiendspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| lv6fiendspfilter(c, duel, e, tp)),
        tp,
        LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_SUMMONED_SKULL)),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
