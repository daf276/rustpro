// Dark Rebellion Xyz Dragon of the Four Heavenly Dragons
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 90091224;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// On the field with Xyz Import to attach Sangan, and Dark Rebellion Xyz Dragon in the Extra Deck to Xyz Summon with the second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (16195942, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn desrescon(duel: &Duel, sg: CardSet, tp: u8, maxc: usize) -> (bool, bool) {
    let ct1 = sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    let ct2 = sg.len() - ct1;
    (ct1 == ct2, ct1 > maxc || ct2 > maxc)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_target_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let ct = g.filter_count(duel, |c, duel| c.is_controler(duel, tp), None);
    let maxc = ct.min(g.len() - ct);
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| desrescon(duel, sg, tp, maxc);
    let tg = aux::select_unselect_group_full(duel, g, 2, g.len(), &rescon, tp, HINTMSG_DESTROY, Some(&rescon), false)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tg), tg.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_rank(duel, &[4])
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_xyz_monster(duel)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn materialchk(duel: &Duel, tp: u8, c: CardId) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    pg.is_empty() || (pg.len() == 1 && pg.contains(c))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(materialchk(duel, tp, c)
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.is_controler(duel, tp) && !c.is_immune_to_effect(duel, e) {
        if !materialchk(duel, tp, c) {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(xyz) = g.get_first() {
            xyz.set_material(duel, Some(c.into()));
            duel.overlay(xyz, c)?;
            if duel.special_summon(xyz, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
                xyz.complete_procedure(duel);
            }
        }
    }
    Ok(())
}
