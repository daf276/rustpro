// The Phantom Knights of Break Sword
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 62709239;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS];
// Doomed Soleret: two Level 3 DARK monsters as materials, and two Level 3 "The Phantom Knights" in the GY to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (83566725, MZONE),
    (83566725, MZONE),
    (83566725, GRAVE),
    (83566725, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let mut g1 = duel
        .select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g2 = duel
        .select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?
        .unwrap_or_default();
    g1.merge(g2);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g1), 2, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let tg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_xyz_summoned(duel)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.has_level(duel)
        && c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_target(
            |c2, duel| spfilter2(c2, duel, c.get_level(duel), e, tp),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            Some(c.into()),
        )
}

fn spfilter2(c: CardId, duel: &Duel, lv: u32, e: EffectId, tp: u8) -> bool {
    c.get_level(duel) == lv && c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_target(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g1 = duel
        .select_target(tp, |c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    let Some(tc1) = g1.get_first() else { return Ok(false) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let lv = tc1.get_level(duel);
    let g2 = duel
        .select_target(
            tp,
            |c, duel| spfilter2(c, duel, lv, e, tp),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            1,
            Some(tc1.into()),
        )?
        .unwrap_or_default();
    g1.merge(g2);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g1), 2, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let tg = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let g = tg.filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    let ct = g.len() as i32;
    if ft > 0 && ct <= ft && (ct == 1 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()) {
        for tc in g {
            duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_LEVEL);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            e1.set_value(duel, 1);
            tc.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
    }
    let e2 = duel.create_effect(e.get_handler(duel));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_target_range(duel, 1, 0);
    e2.set_target_card(duel, |duel, _, c| !c.is_attribute(duel, ATTRIBUTE_DARK));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}
