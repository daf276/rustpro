// Neo Galaxy-Eyes Photon Dragon - Photon Howling
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 28331069;

pub const LISTED_SERIES: &[u16] = &[SET_PHOTON];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(altmatfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

// In the Extra Deck, Xyz Summoned with three Galaxy-Eyes Photon Dragons (Level 8) or over Neo Galaxy-Eyes Photon Dragon (Rank 8),
// with another Galaxy-Eyes Photon Dragon in the Deck to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (93717133, MZONE),
    (93717133, MZONE),
    (93717133, MZONE),
    (39272762, MZONE),
    (93717133, DECK),
    (93717133, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, spattachtg);
    e1.set_operation(duel, spattachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::detach_from_self::<3, 3>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn altmatfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ.into(), tp) && c.is_rank(duel, &[8]) && c.is_faceup(duel)
}

fn spcheck(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    mmz_chk && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn attachcheck(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_xyz_material(duel, Some(e.get_handler(duel)), tp, REASON_EFFECT)
}

fn spattachfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_PHOTON]) && c.is_monster(duel) && (spcheck(c, duel, e, tp, mmz_chk) || attachcheck(c, duel, e, tp))
}

fn spattachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spattachfilter(c, duel, e, tp, mmz_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spattachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(
        tp,
        |c, duel| spattachfilter(c, duel, e, tp, mmz_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    let b1 = spcheck(sc, duel, e, tp, mmz_chk);
    let b2 = attachcheck(sc, duel, e, tp) && c.is_relate_to_effect(duel, e);
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (b2, aux::stringid(ID, 5))])? {
        Some(1) => {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
        Some(2) => duel.overlay_full(c, sc, true)?,
        _ => {}
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, self_exc: Option<CardId>) -> bool {
    c.is_xyz_monster(duel)
        && c.is_releasable_by_effect(duel)
        && duel.is_existing_matching_card(
            CardId::is_negatable,
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(CardSet::from_iter([Some(c), self_exc].into_iter().flatten())),
        )
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|sc, duel| xyzfilter(sc, duel, Some(c)), a.tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(CardId::is_negatable, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), g.len().saturating_sub(1) as u8, a.tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then_some(c);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let sc = duel.select_matching_card(
        tp,
        |sc, duel| xyzfilter(sc, duel, exc),
        tp,
        LOCATION_MZONE,
        0,
        1,
        1,
        exc.map(CardSet::from),
    )?;
    duel.hint_selection(sc);
    if duel.release(sc, REASON_EFFECT)? > 0 {
        let g = duel.get_matching_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc.map(CardSet::from));
        for tc in g {
            tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
        }
    }
    Ok(())
}
