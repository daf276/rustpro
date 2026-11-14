// THE Star Ham
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 15123983;

// In the Extra Deck and Graveyard, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Atlantean Pikeman (Level 2),
// with Sangan in hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (16449363, MZONE),
    (16449363, MZONE),
    (26976414, MZONE),
    (26976414, MZONE),
    (26202165, HAND),
    (26202165, HAND),
    (26202165, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, gyspcon);
    e1.set_cost(duel, Cost::discard);
    e1.set_target(duel, gysptg);
    e1.set_operation(duel, gyspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::discard_full(duel, a, chk, aux::TRUE, false, (2, 2)));
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
}

fn gyspcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_synchro_summoned(duel) && c.get_material_count(duel) > 0
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sync: CardId) -> bool {
    sync.is_reason_card(duel, c)
        && c.is_reason(duel, REASON_SYNCHRO)
        && c.is_reason(duel, REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let sync = e.get_handler(duel);
    stock::special_summon_target(duel, a, chk, |c, duel| gyspfilter(c, duel, e, tp, sync), LOCATION_GRAVE)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = duel
        .select_matching_card(tp, |sc, duel| gyspfilter(sc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
        && !sc.is_type(duel, TYPE_TUNER)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}

pub(super) fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
