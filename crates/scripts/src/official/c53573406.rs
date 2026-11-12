// Masked Chameleon
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 53573406;

// Copies in the hand to Normal Summon, and T.G. Striker, a monster with 0 DEF, in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (1315120, GRAVE),
    (1315120, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_condition(duel, sumcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

fn counterfilter(duel: &Duel, c: CardId) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 5)
}

fn sumcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = e.get_handler(duel);
    let fid = i64::from(c.get_field_id(duel));
    e.set_label(duel, fid);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_label(duel, fid);
    e1.set_target_spsummon(duel, sumlimit);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    aux::add_temp_lizard_check(duel, c, tp);
    Ok(false)
}

fn sumlimit(duel: &Duel, e: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, se: Option<EffectId>) -> bool {
    unported!(duel, se.is_none());
    se.is_some_and(|se| e.get_label(duel) != se.get_label(duel)) && !c.is_location(duel, LOCATION_EXTRA)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_defense_below(duel, 0) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
