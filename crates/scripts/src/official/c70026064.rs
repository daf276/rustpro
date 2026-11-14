// Bujin Hiruko
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, PZONE};

const ID: u32 = 70026064;

pub const LISTED_SERIES: &[u16] = &[SET_BUJIN];
// In the Pendulum Zone, with Bujintei Kagutsuchi and Bujinki Amaterasu (Bujin Xyz Monsters) face-up on the field and Bujintei
// Tsukuyomi and Bujintei Susanowo (other Bujin Xyz Monsters) in the Extra Deck to be Xyz Summoned.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (1855932, MZONE),
    (1855932, MZONE),
    (68618157, MZONE),
    (68618157, MZONE),
    (73289035, EXTRA),
    (75840616, EXTRA),
];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
        && c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_BUJIN])
        && duel.is_existing_matching_card(
            |sc, duel| filter2(sc, duel, e, tp, c, c.get_code(duel)),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, code: u32) -> bool {
    mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_BUJIN])
        && !c.is_code(duel, code)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter1(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| filter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    let pg = aux::get_must_be_material_group(duel, tp, Some(tc.into()), tp, None, None, REASON_XYZ);
    if tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
        || tc.is_controler(duel, 1 - tp)
        || tc.is_immune_to_effect(duel, e)
        || pg.len() > 1
        || (pg.len() == 1 && !pg.contains(tc))
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let code = tc.get_code(duel);
    let g = duel.select_matching_card(tp, |sc, duel| filter2(sc, duel, e, tp, tc, code), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        sc.set_material(duel, Some(tc.into()));
        duel.overlay(sc, tc)?;
        duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
        sc.complete_procedure(duel);
    }
    Ok(())
}
