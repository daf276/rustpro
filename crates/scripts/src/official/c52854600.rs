// Tenyi Spirit - Suruya
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 52854600;

pub const LISTED_SERIES: &[u16] = &[SET_TENYI];
// A copy on the field and in the hand with Mystery Shell Dragon (a Level 4 Wyrm Normal Monster), Draco Berserker of the Tenyi
// (Level 8) to Synchro Summon with them, and Monk of the Tenyi in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (18108166, MZONE),
    (18108166, MZONE),
    (5041348, EXTRA),
    (5041348, EXTRA),
    (5041348, EXTRA),
    (32519092, EXTRA),
    (32519092, EXTRA),
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
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, extraspcon);
    e2.set_target(duel, extrasptg);
    e2.set_operation(duel, extraspop);
    c.register_effect(duel, e2);
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_race(duel, RACE_WYRM) || c.is_non_effect_monster(duel)) && c.is_faceup(duel)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(selfspconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn extraspcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && a.r & REASON_SYNCHRO == REASON_SYNCHRO
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_race(duel, RACE_WYRM))
}

fn extraspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TENYI])
        && c.is_link_monster(duel)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn extrasptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| extraspfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn extraspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = duel
        .select_matching_card(tp, |c, duel| extraspfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3312);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    duel.break_effect()?;
    if let Some(sc) = sc {
        let lp = duel.get_lp(tp) - sc.get_link(duel) as i32 * 1000;
        duel.set_lp(tp, lp);
    }
    Ok(())
}
