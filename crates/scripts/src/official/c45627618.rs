// Odd-Eyes Rebellion Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 45627618;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub const PENDULUM_LEVEL: u32 = 7;

// In the Pendulum Zone with Pendulum Monsters in the Deck to place, and on the field with a Pendulum Zone card to destroy when it is
// destroyed by the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (72181263, DECK),
    (72181263, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    // Needs an Xyz Summon with an Xyz Monster as material, which no ported card makes.
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, pctg);
    e1.set_operation(duel, pcop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_condition(duel, pencon);
    e4.set_target(duel, pentg);
    e4.set_operation(duel, penop);
    c.register_effect(duel, e4);
}

fn pcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn pctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.check_pendulum_zones(tp) && duel.is_existing_matching_card(pcfilter, tp, LOCATION_DECK, 0, 1, None))
}

fn pcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) || !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, pcfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_xyz_summoned(duel) && a.e.get_label(duel) == 1
}

fn levelfilter() -> impl Fn(CardId, &Duel) -> bool {
    aux::faceup_filter(|c, duel| c.is_level_below(duel, 7))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let g = duel.get_matching_group(levelfilter(), tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 1000);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(levelfilter(), tp, 0, LOCATION_MZONE, None);
    let ct = duel.destroy(g, REASON_EFFECT)?;
    if duel.damage(1 - tp, ct as u32 * 1000, REASON_EFFECT)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 2);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let xyz = c.get_material(duel).is_exists(
        duel,
        |mc, duel| mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, PLAYER_NONE),
        1,
        None,
    );
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, i64::from(xyz));
    }
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.r & (REASON_EFFECT + REASON_BATTLE) != 0 && c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_PZONE, 0) > 0);
    }
    let g = duel.get_field_group(tp, LOCATION_PZONE, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_PZONE, 0);
    if duel.destroy(g, REASON_EFFECT)? != 0 && a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        duel.move_to_field(a.e.get_handler(duel), tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
