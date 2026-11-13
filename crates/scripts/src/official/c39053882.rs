// V Umbral Horror
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 39053882;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and Graveyard, with Thousand Umbral Horror in the Deck to add, and Number 104: Masquerade (Rank 4) on the field to
// Xyz Summon Number C104: Umbral Horror Masquerade (Rank 5) with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (2932235, DECK),
    (2932235, DECK),
    (2061963, MZONE),
    (2061963, MZONE),
    (49456901, EXTRA),
    (49456901, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_UMBRAL_HORROR]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn numberspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, rank: u32) -> bool {
    c.is_set_card(duel, &[SET_NUMBER])
        && c.is_xyz_monster(duel)
        && c.is_rank(duel, &[rank])
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn masqueradefilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
        && c.is_set_card(duel, &[SET_MASQUERADE])
        && c.is_xyz_monster(duel)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| numberspfilter(sc, duel, e, tp, c, c.get_rank(duel) + 1),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && masqueradefilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| masqueradefilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| masqueradefilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_controler(duel, tp) && !tc.is_immune_to_effect(duel, e) {
        let pg = aux::get_must_be_material_group(duel, tp, Some(tc.into()), tp, None, None, REASON_XYZ);
        if pg.len() > 1 || (pg.len() == 1 && !pg.contains(tc)) {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let rank = tc.get_rank(duel) + 1;
        let g = duel.select_matching_card(
            tp,
            |c, duel| numberspfilter(c, duel, e, tp, tc, rank),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        if let Some(xyz) = g.get_first() {
            xyz.set_material(duel, Some(tc.into()));
            duel.overlay(xyz, tc)?;
            if duel.special_summon(xyz, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? == 0 {
                return Ok(());
            }
            xyz.complete_procedure(duel);
        }
    }
    Ok(())
}
