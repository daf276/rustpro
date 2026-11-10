// Shooting Star Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 24696097;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[CARD_STARDUST_DRAGON])
    }
}

pub const MATERIAL: &[u32] = &[CARD_STARDUST_DRAGON];
pub const LISTED_NAMES: &[u32] = &[CARD_STARDUST_DRAGON];

// On the field against the opponent's Elfnote Lucina to attack and Raigeki to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        duel.is_able_to_enter_bp() && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) >= 5
    });
    e1.set_operation(duel, mtop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, super::c44508094::condition);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
    });
    e3.set_target(duel, datg);
    e3.set_operation(duel, daop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_FIELD);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_range(duel, LOCATION_REMOVED);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, sumtg);
    e4.set_operation(duel, sumop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_MATERIAL_CHECK);
    e5.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e5);
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.confirm_decktop(tp, 5)?;
    let g = duel.get_decktop_group(tp, 5);
    let ct = g.filter_count(duel, |c, duel| c.is_type(duel, TYPE_TUNER), None);
    duel.shuffle_deck(tp)?;
    if ct > 1 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, ct as i32 - 1);
        c.register_effect(duel, e1);
    } else if ct == 0 {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_CANNOT_ATTACK);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_relate_to_effect(duel, re) && rc.is_destructable(duel) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && duel.negate_effect(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn datg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, e) = (a.e.get_handler(duel), a.e);
    let Some(attacker) = duel.get_attacker() else { return Ok(false) };
    if let Some(chkc) = chkc {
        return Ok(chkc == attacker);
    }
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && attacker.is_can_be_effect_target(duel, e) && !c.is_status(duel, STATUS_CHAINING));
    }
    duel.set_target_card(attacker);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn daop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.remove(c, POS_FACEUP, REASON_EFFECT)? != 0 {
        duel.negate_attack()?;
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 0);
    }
    Ok(())
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.has_flag_effect(duel, ID)
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_type(duel, TYPE_TUNER), 2, None)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_MULTIPLE_TUNERS);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD & !RESET_TOFIELD | RESET_PHASE | PHASE_END);
        c.register_effect(duel, e1);
    }
}
