// Shooting Quasar Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 35952884;
const SHOOTING_STAR_DRAGON: u32 = 24696097;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[SHOOTING_STAR_DRAGON];

// On the field against the opponent's Raigeki and Dark Hole, with Shooting Star Dragon in the Extra Deck to Special Summon.
// The multi attack needs a Synchro Summon from three Synchro Monsters, which random play never reaches.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (SHOOTING_STAR_DRAGON, EXTRA),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(SHOOTING_STAR_DRAGON),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    c.add_must_be_synchro_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label(duel) > 0
    });
    e2.set_operation(duel, mtop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_count_limit(duel, 1, 0);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| {
        !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
    });
    e3.set_target(duel, stock::negate_destroy_target);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_position(duel, POS_FACEUP) && c.is_previous_location(duel, LOCATION_ONFIELD)
    });
    e4.set_target(duel, sumtg);
    e4.set_operation(duel, sumop);
    c.register_effect(duel, e4);
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let ct = c
        .get_material(duel)
        .filter_count(duel, |mc, duel| !mc.is_type(duel, TYPE_TUNER), None);
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, ct as i64);
    }
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = a.e.get_label(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    e1.set_value(duel, (ct - 1) as i32);
    c.register_effect(duel, e1);
    Ok(())
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, SHOOTING_STAR_DRAGON)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tg) = duel.get_first_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, None) {
        duel.special_summon(tg, 0, tp, tp, false, true, POS_FACEUP)?;
    }
    Ok(())
}
