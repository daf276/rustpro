// Hyper-Rank-Up-Magic Ultimate Force
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 87818869;

pub const LISTED_SERIES: &[u16] = &[SET_UTOPIC, SET_NUMBER, SET_NUMBER_99];
// Utopia on the field and Number 99: Utopic Dragon in the Extra Deck, which gains the granted effect, with Elfnote Lucina on the
// opponent's field and Kuriboh in its hand to activate monster effects in the Battle Phase. No Rank 10 or higher "Number" Xyz
// Monster ported has a rum_limit, so that check is left out.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (84013237, MZONE),
    (84013237, MZONE),
    (51543904, EXTRA),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (40640057, HAND | OPPONENT),
    (40640057, HAND | OPPONENT),
    (40640057, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(51543904), Check::ResolvesGranted(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_XMATERIAL + EFFECT_TYPE_QUICK_F);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    (pg.is_empty() || pg.contains(c))
        && c.is_rank_below(duel, 9)
        && c.is_set_card(duel, &[SET_UTOPIC])
        && c.is_original_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && c.is_rank_above(duel, 10)
        && c.is_set_card(duel, &[SET_NUMBER])
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
            && duel.is_existing_target(|c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_controler(duel, tp) && !tc.is_immune_to_effect(duel, e) {
        let pg = aux::get_must_be_material_group(duel, tp, Some(tc.into()), tp, None, None, REASON_XYZ);
        // The Lua's `pg:IsContains(c)` reads the undefined global `c`, which errors.
        unported!(duel, !pg.is_empty());
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        let Some(xyz) = g.get_first() else { return Ok(()) };
        xyz.set_material(duel, Some(tc.into()));
        duel.overlay(xyz, tc)?;
        if duel.special_summon(xyz, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
            xyz.complete_procedure(duel);
            let c = e.get_handler(duel);
            if c.is_relate_to_effect(duel, e) {
                c.cancel_to_grave(duel);
                duel.overlay(xyz, c)?;
            }
        }
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.is_battle_phase()
        && a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && a.e.get_handler(duel).is_set_card(duel, &[SET_NUMBER_99])
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let og = c.get_overlay_group(duel);
    if c.is_relate_to_effect(duel, e) && !og.is_empty() && duel.send_to_grave(og, REASON_EFFECT)? == og.len() as i32 {
        let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
        if !g.is_empty() {
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
