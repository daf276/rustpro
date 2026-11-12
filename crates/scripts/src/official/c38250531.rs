// Crimson Knight Vampire Bram
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 38250531;

// In the Extra Deck, Xyz Summoned from Level 5 Zombies (Vampire Lord, Shadow Ghoul), with Sangan in the opponent's Graveyard to revive
// and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (53839837, MZONE),
    (53839837, MZONE),
    (30778711, MZONE),
    (30778711, MZONE),
    (26202165, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_ZOMBIE, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, spreg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_condition(duel, spcon2);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_target_card(duel, ftarget);
        e1.set_label(duel, i64::from(tc.get_field_id(duel)));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn ftarget(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_label(duel) != i64::from(c.get_field_id(duel))
}

fn spreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if a.rp != tp && c.is_reason(duel, REASON_DESTROY) && c.is_previous_location(duel, LOCATION_ONFIELD) {
        let turn = duel.get_turn_count();
        e.set_label(duel, i64::from(turn + 1));
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 2);
    }
    Ok(())
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let label = a.e.get_label_object(duel).map_or(0, |e| e.get_label(duel));
    label == i64::from(duel.get_turn_count()) && a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
