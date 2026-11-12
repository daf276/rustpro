// Salamangreat Burst Gryphon
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 10140443;

pub const LISTED_NAMES: &[u32] = &[ID];

// A copy on the field and one in the Extra Deck, with Level 1 and 5 FIRE monsters in the Graveyard to Special Summon. The Reincarnation
// Summon (using a copy as material) is out of reach: it needs the copy's Level lowered first.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA), (13179332, GRAVE), (17192817, GRAVE), (14977074, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    aux::enable_check_reincarnation(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, effcon);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reincarnation_summoned(duel) && c.is_synchro_summoned(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    let (reset, mut reset_ct, mut turn_ct) = (RESET_PHASE | PHASE_STANDBY, 1, 0);
    if duel.is_phase(PHASE_STANDBY) {
        reset_ct = 2;
        turn_ct = duel.get_turn_count();
    }
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, delayspcon);
    e1.set_operation(duel, delayspop);
    e1.set_label(duel, i64::from(turn_ct));
    e1.set_reset_full(duel, reset, reset_ct);
    duel.register_effect(e1, tp);
    aux::register_client_hint_full(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)), reset, reset_ct);
    Ok(())
}

fn delayspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn delayspcon(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let label = e.get_label(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && (label == 0 || label != i64::from(duel.get_turn_count()))
        && duel.is_existing_matching_card(
            aux::necro_valley_filter(|c, duel| delayspfilter(c, duel, e, tp)),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn delayspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| delayspfilter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.select(duel, tp, 1, 1, None)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lvl: u32) -> bool {
    c.get_level(duel) < lvl
        && c.is_level_below(duel, 7)
        && c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let lvl = c.get_level(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp, lvl));
    }
    if chk == 0 {
        return Ok(c.has_level(duel)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp, lvl), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, lvl), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, card| card.is_attribute_except(duel, ATTRIBUTE_FIRE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && c.has_level(duel)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -(tc.get_original_level(duel) as i32));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
