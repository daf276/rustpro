// Barian's Hope
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 75518330;

// In the hand and Graveyard, with Umbral Horror monsters in the hand and on the field to Special Summon and change Levels, and a
// Number C104 on the field with Numbers (Number C104 and Number 83: Galaxy Queen) in the Extra Deck and Graveyard to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (2932235, HAND),
    (3064783, GRAVE),
    (76548531, MZONE),
    (49456901, MZONE),
    (42396591, EXTRA),
    (48928529, GRAVE),
    (2932235, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_UMBRAL_HORROR]) || (c.is_set_card(duel, &[SET_MASQUERADE]) && c.is_xyz_monster(duel)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_LVCHANGE, None, 1, tp, 0);
    Ok(false)
}

fn umbralfilter(c: CardId, duel: &Duel, umbral_group: CardSet) -> bool {
    let level = c.get_level(duel);
    c.is_set_card(duel, &[SET_UMBRAL_HORROR])
        && c.has_level(duel)
        && c.is_faceup(duel)
        && umbral_group.is_exists(duel, |uc, duel| !uc.is_level(duel, &[level]), 1, Some(c.into()))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if g.is_empty() || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let umbral_group = duel.get_matching_group(
        |c, duel| c.is_set_card(duel, &[SET_UMBRAL_HORROR]) && c.has_level(duel) && c.is_faceup(duel),
        tp,
        LOCATION_MZONE,
        0,
        None,
    );
    if !umbral_group.is_empty()
        && duel.is_existing_matching_card(
            |c, duel| umbralfilter(c, duel, umbral_group),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            0,
            1,
            None,
        )
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
        let sc = duel.select_matching_card(
            tp,
            |c, duel| umbralfilter(c, duel, umbral_group),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        let Some(sc) = sc.get_first() else { return Ok(()) };
        duel.hint_selection(sc);
        let c = e.get_handler(duel);
        let level = sc.get_level(duel);
        duel.break_effect()?;
        for tc in umbral_group {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL_FINAL);
            e1.set_value(duel, level as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_set_card(duel, &[SET_NUMBER]) && c.is_xyz_monster(duel) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_xyz_monster(duel)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |ac, duel| attachfilter(ac, duel, tp, c),
            tp,
            LOCATION_EXTRA | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| xyzfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| xyzfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_xyz_monster(duel) && !tc.is_immune_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let filter = aux::necro_valley_filter(|c, duel| attachfilter(c, duel, tp, tc));
        let Some(sc) = duel
            .select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        if sc.is_location(duel, LOCATION_EXTRA) {
            duel.confirm_cards(1 - tp, sc)?;
        } else {
            duel.hint_selection(sc);
        }
        duel.overlay(tc, sc)?;
    }
    Ok(())
}
