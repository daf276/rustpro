// Sky Striker Mobilize - Linkage!
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, SZONE};

const ID: u32 = 9726840;

pub const LISTED_SERIES: &[u16] = &[SET_SKY_STRIKER_ACE];
// In the hand with a Spell on the field to send (no monsters), Sky Striker Ace Link Monsters in the Extra Deck, and a DARK and
// a LIGHT Sky Striker Ace in the GY for the ATK gain.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (98338152, GRAVE),
    (8491308, EXTRA),
    (63288573, EXTRA),
    (98462037, EXTRA),
    (26077387, GRAVE),
    (37351133, GRAVE),
    (52340444, SZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_condition(duel, tgcon);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_MMZONE, 0) == 0
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_able_to_grave(duel) && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER_ACE])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex_full(tp, tp, Some(tc.into()), c, ZONES_EMZ) > 0
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| tgfilter(c, duel, e, tp),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            Some(e.get_handler(duel).into()),
        ));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn attfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SKY_STRIKER_ACE]) && c.is_attribute(duel, att)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_SKY_STRIKER_ACE])
        });
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| tgfilter(tc, duel, e, tp),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_grave(g, REASON_EFFECT)? < 1 || !tc.is_location(duel, LOCATION_GRAVE) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(sc) = duel
        .select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let both_attributes = |duel: &Duel| {
        [ATTRIBUTE_DARK, ATTRIBUTE_LIGHT]
            .into_iter()
            .all(|att| duel.is_existing_matching_card(|c, duel| attfilter(c, duel, att), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None))
    };
    if duel.special_summon_full(sc, 0, tp, tp, false, false, POS_FACEUP, ZONES_EMZ)? > 0 && both_attributes(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect_full(duel, e1, true);
    }
    Ok(())
}
