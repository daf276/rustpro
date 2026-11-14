// Mathmech Superfactorial
use crate::keys;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, SZONE};

const ID: u32 = 87804365;

pub const LISTED_SERIES: &[u16] = &[SET_MATHMECH];

// Set and in the hand, with Level 4 Mathmech Tuners (Sigma, Diameter) and non-Tuners (Addition, Subtraction) in the
// Graveyard for Geomathmech Magma, Geomathmech Final Sigma, Primathmech Alembertian and Primathmech Laplacian.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (27182739, GRAVE),
    (17946349, GRAVE),
    (80965043, GRAVE),
    (16360142, GRAVE),
    (15248594, EXTRA),
    (42632209, EXTRA),
    (85692042, EXTRA),
    (88021907, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target::<TYPE_SYNCHRO>);
    e1.set_operation(duel, operation::<TYPE_SYNCHRO>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_target(duel, target::<TYPE_XYZ>);
    e2.set_operation(duel, operation::<TYPE_XYZ>);
    c.register_effect(duel, e2);
}

// The chkfun of s.filter and s.rescon: Card.IsSynchroSummonable or Card.IsXyzSummonable with `mg` as the materials.
fn summonable<const MONTYPE: u32>(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    let count = mg.len() as u16;
    match MONTYPE {
        TYPE_SYNCHRO => c.is_synchro_summonable_full(duel, None, Some(mg), count, count),
        _ => c.is_xyz_summonable_full(duel, None, Some(mg), count, count),
    }
}

fn relfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn matfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MATHMECH]) && c.is_can_be_effect_target(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn filter<const MONTYPE: u32>(c: CardId, duel: &Duel, mg: Option<CardSet>, tp: u8, chk: bool) -> bool {
    c.is_set_card(duel, &[SET_MATHMECH])
        && c.is_type(duel, MONTYPE)
        && (!chk || duel.get_location_count_from_ex(tp, tp, mg, c) > 0)
        && mg.is_none_or(|mg| summonable::<MONTYPE>(c, duel, mg))
}

fn rescon<const MONTYPE: u32>(exg: CardSet) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, mg, c| {
        let (ok, stop) = aux::dncheck(duel, sg, mg, c);
        (
            ok && exg.is_exists(duel, |sc, duel| summonable::<MONTYPE>(sc, duel, sg), 1, None),
            stop,
        )
    }
}

fn target<const MONTYPE: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let exg = duel.get_matching_group(|c, duel| filter::<MONTYPE>(c, duel, None, tp, false), tp, LOCATION_EXTRA, 0, None);
    let cancelcon = rescon::<MONTYPE>(exg);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp)
            && chkc.is_location(duel, LOCATION_GRAVE)
            && chkc.is_set_card(duel, &[SET_MATHMECH])
            && chkc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && cancelcon(duel, chkc.into(), CardSet::default(), None).0);
    }
    let mg = duel.get_matching_group(|c, duel| matfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    let count = duel.get_location_count(tp, LOCATION_MZONE);
    let blue_eyes_spirit = duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some();
    let min = count.min(if blue_eyes_spirit { 1 } else { mg.len() as i32 }).min(1);
    let ft = duel.get_location_count(tp, LOCATION_MZONE).min(3);
    if chk == 0 {
        return Ok(min > 0
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, mg, min as usize, ft as usize, &cancelcon));
    }
    let sg = aux::select_unselect_group_full(
        duel,
        mg,
        min.max(0) as usize,
        ft.max(0) as usize,
        &cancelcon,
        tp,
        HINTMSG_SPSUMMON,
        Some(&cancelcon),
        false,
    )?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn operation<const MONTYPE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel
        .get_target_cards(e)
        .unwrap_or_default()
        .filter(duel, |c, duel| relfilter(c, duel, e, tp), None);
    if duel.get_location_count(tp, LOCATION_MZONE) < g.len() as i32
        || g.is_empty()
        || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() && g.len() > 1
    {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for tc in g {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    duel.break_effect()?;
    let syng = duel.get_matching_group(|c, duel| filter::<MONTYPE>(c, duel, Some(g), tp, true), tp, LOCATION_EXTRA, 0, None);
    if !syng.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let Some(sc) = syng.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        let count = g.len() as u16;
        if MONTYPE == TYPE_SYNCHRO {
            duel.set_script_state(keys::SYNCHRO_SEND, 5);
            duel.synchro_summon_full(tp, sc, None, Some(g), count, count)?;
        } else {
            duel.xyz_summon_full(tp, sc, Some(g), None, count, count)?;
        }
    }
    Ok(())
}
