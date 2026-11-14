// Performage Trapeze Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 17016362;

pub const LISTED_SERIES: &[u16] = &[SET_PERFORMAGE];
// Xyz Summoned from the Extra Deck with Level 4 Spellcasters (Mystical Elf) that it also gives a second attack, destroyed by the
// opponent's Raigeki to Special Summon Performage Trick Clown from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (15025844, MZONE),
    (15025844, MZONE),
    (67696066, DECK),
    (67696066, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(67696066)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e2.set_condition(duel, mtcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, mttg);
    e2.set_operation(duel, mtop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, _: u32, _: u8, _: Option<CardId>) -> u32 {
    let atk = e.get_handler(duel).get_attack(duel);
    if i64::from(val) <= i64::from(atk) { 0 } else { val }
}

fn mtcon(duel: &Duel, _: &Args) -> bool {
    duel.is_phase(PHASE_MAIN1) && duel.is_able_to_enter_bp()
}

fn mtfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_has_effect(duel, EFFECT_EXTRA_ATTACK).is_none()
}

fn mttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let turnp = duel.get_turn_player();
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, turnp) && mtfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(mtfilter, turnp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, mtfilter, turnp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    let fid = c.get_field_id(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        tc.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, i64::from(fid), 0);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_count_limit(duel, 1, 0);
        e2.set_label(duel, i64::from(fid));
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_condition(duel, descon);
        e2.set_operation(duel, desop);
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, tp);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return false;
    };
    tc.get_flag_effect_label(duel, ID) == Some(a.e.get_label(duel))
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_reason(duel, REASON_BATTLE) || (a.rp != tp && c.is_reason(duel, REASON_DESTROY) && c.is_previous_controler(duel, tp))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PERFORMAGE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
