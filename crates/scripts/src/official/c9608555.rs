// Salamangreat Blaze Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 9608555;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
pub const LISTED_NAMES: &[u32] = &[ID];

// A copy on the field and two in the Extra Deck, to Reincarnation Summon with the field copy as material, and Xyz Summoned with two
// of four Level 4 monsters, with the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (5628232, MZONE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    aux::enable_check_reincarnation(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_battle_phase() && a.e.get_handler(duel).get_overlay_count(duel) == 0
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
            && !c.is_reason(duel, REASON_REPLACE)
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    Ok(true)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, pg: CardSet) -> bool {
    c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_SALAMANGREAT])
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && (pg.is_empty() || pg.is_contains(mc))
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok(
            pg.len() <= 1 && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c, pg), tp, LOCATION_EXTRA, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.is_controler(duel, tp) && !c.is_immune_to_effect(duel, e)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c, pg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, Some(c.into()));
    duel.overlay(sc, c)?;
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reincarnation_summoned(duel) && c.is_xyz_summoned(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
