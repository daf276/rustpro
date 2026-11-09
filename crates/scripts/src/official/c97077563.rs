// Call of the Haunted
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, OPPONENT, SZONE};

const ID: u32 = 97077563;

// Psychic Kappa, a Normal Monster, in the Graveyard to revive, and Mystical Space Typhoon in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (7892180, GRAVE), (5318639, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_LEAVE_FIELD_P);
    e2.set_operation(duel, |duel, a| {
        let disabled = a.e.get_handler(duel).is_disabled(duel);
        a.e.set_label(duel, disabled.into());
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_operation(duel, mondesop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_condition(duel, selfdescon);
    e4.set_operation(duel, |duel, a| {
        duel.destroy(a.e.get_handler(duel), REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?
        && c.is_relate_to_effect(duel, e)
    {
        c.set_card_target(duel, tc);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn mondesop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_label_object(duel).is_some_and(|e2| e2.get_label(duel) != 0) {
        return Ok(());
    }
    if let Some(tc) = a.e.get_handler(duel).get_first_card_target(duel)
        && tc.is_location(duel, LOCATION_MZONE)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn selfdescon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.is_contains(tc)) && tc.is_reason(duel, REASON_DESTROY))
}
