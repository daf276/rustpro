// Performapal Odd-Eyes Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, PZONE};

const ID: u32 = 82224646;

// Two on the field and one in the Graveyard and hand, with Performapal Skullcrobat Joker (Level 4, not a Tuner) in a Pendulum Zone
// and on the field, and Level 6 Synchros in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (ID, HAND),
    (40318957, PZONE),
    (40318957, MZONE),
    (19974580, EXTRA),
    (93172951, EXTRA),
    (64880894, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, tntg);
    e1.set_operation(duel, tnop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    e2.set_condition(duel, rmcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, sctg);
    e4.set_operation(duel, scop);
    c.register_effect(duel, e4);
}

fn tnfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && (c.is_set_card(duel, &[SET_PERFORMAPAL]) || c.is_set_card(duel, &[SET_ODD_EYES]))
        && c.has_level(duel)
        && (!c.is_type(duel, TYPE_TUNER) || c.is_level_above(duel, 2))
}

fn tntg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tnfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tnfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, tnfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn tnop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_CHANGE_LEVEL);
        e2.set_value(duel, 1);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_summon_location(duel, LOCATION_EXTRA) && c.is_reason(duel, REASON_MATERIAL) && c.is_reason(duel, REASON_SYNCHRO)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 3)
        && (c.is_set_card(duel, &[SET_PERFORMAPAL]) || c.is_set_card(duel, &[SET_ODD_EYES]))
        && c.is_can_be_special_summoned(duel, e, 182, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn negate(duel: &mut Duel, c: CardId, tc: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    tc.register_effect(duel, e2);
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 182, tp, tp, false, false, POS_FACEUP)?
    {
        negate(duel, c, tc);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn scfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    let mg = CardSet::from_iter([c, mc]);
    c.is_can_be_synchro_material_full(duel, None, None, None)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|sc, duel| scfilter2(sc, duel, tp, mg), tp, LOCATION_EXTRA, 0, 1, None)
}

fn scfilter2(c: CardId, duel: &Duel, tp: u8, mg: CardSet) -> bool {
    duel.get_location_count_from_ex(tp, tp, Some(mg), c) > 0 && c.is_synchro_summonable(duel, None, Some(mg))
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && chkc.is_controler(duel, tp) && scfilter1(chkc, duel, e, tp, c));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|tc, duel| scfilter1(tc, duel, e, tp, c), tp, LOCATION_PZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |tc, duel| scfilter1(tc, duel, e, tp, c), tp, LOCATION_PZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || !duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        return Ok(());
    }
    negate(duel, c, tc);
    duel.special_summon_complete()?;
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = CardSet::from_iter([c, tc]);
    let g = duel.get_matching_group(|sc, duel| scfilter2(sc, duel, tp, mg), tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, None, Some(mg))?;
        }
    }
    Ok(())
}
