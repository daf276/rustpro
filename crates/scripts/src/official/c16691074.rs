// Odd-Eyes Absolute Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 16691074;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck to Xyz Summon from Level 7 Normal Monsters, on the field with the opponent's Raigeki to destroy it and an
// Odd-Eyes Fusion Monster (Odd-Eyes Vortex Dragon) in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (53262004, EXTRA),
    (6368038, MZONE),
    (36996508, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_MZONE) && c.is_xyz_summoned(duel)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ODD_EYES]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.negate_attack()? || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter1(c, duel, e, tp));
    let g1 = duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, None);
    if !g1.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g2 = g1.select(duel, tp, 1, 1, None)?;
        duel.special_summon(g2, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ODD_EYES])
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
