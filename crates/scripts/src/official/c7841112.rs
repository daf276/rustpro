// Majestic Star Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 7841112;
pub const MATERIAL: &[u32] = &[21159309, CARD_STARDUST_DRAGON];
pub const LISTED_NAMES: &[u32] = &[21159309, CARD_STARDUST_DRAGON];

// Majestic Dragon, Stardust Dragon and Mokey Mokey (Level 1) as materials; two copies on the field copy the Cannon Soldiers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (21159309, MZONE),
    (CARD_STARDUST_DRAGON, MZONE),
    (27288416, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (11384280, MZONE | OPPONENT),
    (11384280, MZONE | OPPONENT),
    (11384280, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(11384280, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl MajesticProcedure for Procedure {
    const CBT1: bool = true;
    const CBT2: bool = true;
    const CBT3: bool = false;

    fn f1(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_code(duel, 21159309)
    }

    fn f2(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_code(duel, CARD_STARDUST_DRAGON)
    }

    fn f3(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_majestic_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_operation(duel, alop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_MATERIAL_CHECK);
    e5.set_value(duel, Value::MaterialCheck(super::c4891376::valcheck));
    c.register_effect(duel, e5);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && a.ep == 1 - a.tp && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let g = duel.get_matching_group(aux::TRUE, a.tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && duel.negate_activation(a.ev as u8)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    let g = duel.get_matching_group(aux::TRUE, a.tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
        duel.majestic_copy(c, tc);
    }
    Ok(())
}

fn is_copied(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    re.get_owner(duel) == e.get_owner(duel) && !re.is_has_property(duel, EFFECT_FLAG_INITIAL)
}

fn alop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && is_copied(duel, a.e, re)
    {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_target_range(duel, 1, 1);
        e1.set_value(duel, Value::EffectPlayer(|duel, e, re, _| is_copied(duel, e, re)));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_STARDUST_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, Some(e.get_handler(duel).into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && c.is_able_to_extra(duel)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? != 0
        && c.is_location(duel, LOCATION_EXTRA)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
