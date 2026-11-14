// Salamangreat Charge
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 83533296;

// Set and in the hand, with Salamangreat Gazelle, a FIRE monster, in the GY to shuffle and summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (26889158, GRAVE),
    (26889158, GRAVE),
    (26889158, GRAVE),
    (26889158, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(26889158)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_can_be_effect_target(duel, e)
        && (c.is_able_to_deck(duel) || c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_type(duel, TYPE_RITUAL | TYPE_EXTRA)
        && c.get_attack(duel) != c.get_base_attack(duel)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> bool {
    sg.is_exists(duel, |c, duel| spcheck(c, duel, sg, e, tp), 1, None)
}

fn spcheck(c: CardId, duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && sg.filter_count(duel, CardId::is_able_to_deck, Some(c.into())) == 2
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(|c, duel| filter1(c, duel, e, tp), tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| (rescon(duel, sg, e, tp), false);
    let b1 = g.len() >= 3 && duel.get_location_count(tp, LOCATION_MZONE) > 0 && aux::select_unselect_group_check(duel, g, 3, 3, &rescon);
    let b2 = duel.is_existing_matching_card(filter2, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()));
    // The Lua compares the undefined global `chck` with the handler, which is always true.
    if let Some(chkc) = chkc {
        return Ok(e.get_label(duel) == 2 && chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TODECK | CATEGORY_SPECIAL_SUMMON);
        let rg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_TARGET)?;
        duel.set_target_card(rg);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(rg), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_TODECK, Some(rg), 2, tp, 0);
    } else {
        e.set_category(duel, CATEGORY_DESTROY);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let tc = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
        duel.set_operation_info(0, CATEGORY_DESTROY, tc, 1, tp, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.get_label(duel) != 1 {
        return stock::destroy_first_target(duel, a);
    }
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.len() < 3 {
        return Ok(());
    }
    let tdg = aux::select_unselect_group(
        duel,
        g,
        2,
        2,
        &|duel, sg, _, _| (selectcond(duel, sg, g, e, tp), false),
        tp,
        HINTMSG_TODECK,
    )?;
    let g = g.difference(tdg);
    if duel.send_to_deck(tdg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let Some(tc) = g.get_first() else { return Ok(()) };
    if tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let c = e.get_handler(duel);
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn selectcond(duel: &Duel, sg: CardSet, resg: CardSet, e: EffectId, tp: u8) -> bool {
    sg.is_exists(duel, CardId::is_able_to_deck, 2, None)
        && resg.is_exists(
            duel,
            |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            1,
            Some(sg),
        )
}
