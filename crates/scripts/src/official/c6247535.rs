// Borreload eXcharge Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 6247535;

// Xyz Summoned from the Extra Deck with Level 4 DARK Dragons (Koumori Dragon), against the opponent's monsters (Sangan), with a Borrel
// Link Monster (Borrelsword Dragon) in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (67724379, MZONE),
    (67724379, MZONE),
    (67724379, MZONE),
    (85289965, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(85289965)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(mfilter);
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, tgcon);
    e1.set_value(duel, Value::EffectPlayer(tgval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn mfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_DRAGON, Some(lc), sumtype, tp) && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(lc), sumtype, tp)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel)
}

fn tgval(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    re.is_monster_effect(duel) && e.get_handler(duel) != re.get_handler(duel)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.get_attack(duel) > 0 || c.get_defense(duel) > 0)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BORREL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && !tc.is_immune_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -600);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e2);
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_GRAVE,
            0,
            None,
        );
        if !g.is_empty() && duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sc = g.select(duel, tp, 1, 1, None)?.get_first();
            if let Some(sc) = sc
                && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? != 0
            {
                sc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
                let e3 = duel.create_effect(c);
                e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
                e3.set_code(duel, EVENT_PHASE + PHASE_END);
                e3.set_count_limit(duel, 1, 0);
                e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
                e3.set_label_object_group(duel, Some(sc.into()));
                e3.set_condition_mut(duel, rmcon);
                e3.set_operation(duel, rmop);
                duel.register_effect(e3, tp);
            }
        }
    }
    let ge1 = duel.create_effect(c);
    ge1.set_type(duel, EFFECT_TYPE_FIELD);
    ge1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    ge1.set_property(duel, EFFECT_FLAG_OATH + EFFECT_FLAG_PLAYER_TARGET);
    ge1.set_target_range(duel, 1, 0);
    ge1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge1, tp);
    let ge2 = duel.create_effect(c);
    ge2.set_type(duel, EFFECT_TYPE_FIELD);
    ge2.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    ge2.set_property(duel, EFFECT_FLAG_OATH + EFFECT_FLAG_IGNORE_IMMUNE);
    ge2.set_target_range(duel, LOCATION_MZONE, 0);
    ge2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0) {
        return true;
    }
    a.e.reset(duel);
    false
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
