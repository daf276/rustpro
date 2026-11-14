// Donner, Dagger Fur Hire
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 8728498;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];

pub struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp))
    }
}

// In the Extra Deck, Link Summoned from Fur Hire monsters of different Types, with Fur Hire monsters in the hand and Graveyard and an opponent's monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (28004531, MZONE),
    (93850652, MZONE),
    (93850652, HAND),
    (47647354, GRAVE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, { LOCATION_HAND | LOCATION_GRAVE }>,
    );
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_effect_target(duel, e) && (c.is_controler(duel, 1 - tp) || (c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE])))
}

fn desrescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) == 2, false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &desrescon));
    }
    let dg = aux::select_unselect_group(duel, g, 2, 2, &desrescon, tp, HINTMSG_DESTROY)?;
    duel.set_target_card(dg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 2, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let code = c.get_original_code(duel);
    duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, code),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && !c.is_original_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| spcostfilter(c, duel, e, tp), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| spcostfilter(c, duel, e, tp), (1, 1), false, None, None)?;
    if let Some(rc) = g.get_first() {
        e.set_labels(duel, &[i64::from(rc.get_original_code(duel)), i64::from(rc.get_type(duel))]);
        duel.release(rc, REASON_COST)?;
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let &[code, kind] = e.get_labels(duel) else { return Ok(()) };
    let filter = |c: CardId, duel: &Duel| spfilter(c, duel, e, tp, code as u32);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g1 = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(filter),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g1.is_empty()
        && duel.special_summon(g1, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && kind & i64::from(TYPE_LINK) == i64::from(TYPE_LINK)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(aux::necro_valley_filter(filter), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g2 = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(filter),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        if g2.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.special_summon(g2, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
