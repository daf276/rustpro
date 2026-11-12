// Odd-Eyes Rebellion Dragon Overlord
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 98452268;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> =
        Some(|c, duel, _, _| c.is_set_card(duel, &[SET_REBELLION]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel));
    const DESC: u64 = aux::stringid(ID, 0);
}

pub const PENDULUM_LEVEL: u32 = 7;

// In the Pendulum Zone and on the field with D/D Orthros in the other Pendulum Zone to attach, Rebellion Xyz Monsters in the Extra Deck to
// summon, and Odd-Eyes Rebellion Dragon on the field to Xyz Summon over.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, EXTRA),
    (ID, MZONE),
    (72181263, PZONE),
    (16195942, EXTRA),
    (64276752, EXTRA),
    (45627618, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    Pendulum::add_procedure_full(duel, c, false, None);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE);
    e2a.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2a.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e2a.set_value(duel, 2);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_SINGLE);
    e2b.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2b.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
    e3.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e3.set_operation(duel, penop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_REBELLION, SET_THE_PHANTOM_KNIGHTS])
        && c.is_type(duel, TYPE_XYZ)
        && c.is_facedown(duel)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && !c.is_code(duel, ID)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_count(tp, 2)
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && (pg.is_empty() || (pg.len() == 1 && pg.is_contains(c)))
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 2, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if !(pg.is_empty() || (pg.len() == 1 && pg.is_contains(c))) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, Some(c.into()));
    duel.overlay(sc, c)?;
    duel.break_effect()?;
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    sc.complete_procedure(duel);
    let matfilter = |mc: CardId, duel: &Duel| mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT);
    if duel.is_existing_matching_card(matfilter, tp, LOCATION_PZONE, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        let g = duel.select_matching_card(tp, matfilter, tp, LOCATION_PZONE, 0, 1, 1, None)?;
        let Some(matc) = g.get_first() else { return Ok(()) };
        duel.hint_selection(matc);
        duel.break_effect()?;
        duel.overlay(sc, matc)?;
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_rank(duel, &[7]) && mc.is_type(duel, TYPE_XYZ), 1, None)
    {
        c.register_flag_effect_full(
            duel,
            ID,
            RESET_EVENT | RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE),
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 4),
        );
    }
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
