// Dark Tuner Catastrogue
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 15141103;

// In the hand and on the field with Dark Tuner Nightmare Hand to Normal Summon, Overdrive (Level 4) as the non-Tuner to Tribute with
// Dark Diviner (Level 4) in the Extra Deck, and a Level 4 Normal Monster as the other Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (51546708, HAND),
    (2311603, MZONE),
    (2311603, MZONE),
    (2311603, MZONE),
    (31919988, EXTRA),
    (31919988, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(31919988),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_normal_summoned(duel));
    e2.set_target(duel, darksynchtg);
    e2.set_operation(duel, darksynchop);
    c.register_effect(duel, e2);
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_TUNER]) && c.is_summonable(duel, true, None)
}

pub fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    Ok(false)
}

pub fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}

fn nontunerfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32, mc: CardId) -> bool {
    !c.is_type(duel, TYPE_TUNER)
        && c.is_releasable(duel)
        && c.is_faceup(duel)
        && c.has_level(duel)
        && c.is_level_below(duel, lv.saturating_sub(1))
        && duel.is_existing_matching_card(
            |sc, duel| darksyncfilter(sc, duel, e, tp, lv - c.get_level(duel), Some(CardSet::from_iter([c, mc]))),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

pub fn darksyncfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32, mg: Option<CardSet>) -> bool {
    c.is_synchro_monster(duel)
        && c.is_level(duel, &[lv])
        && duel.get_location_count_from_ex(tp, tp, mg, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

pub fn darksynchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_SYNCHRO);
        return Ok(pg.is_empty()
            && c.is_releasable(duel)
            && duel.is_existing_matching_card(
                |sc, duel| nontunerfilter(sc, duel, e, tp, c.get_level(duel), c),
                tp,
                LOCATION_MZONE,
                0,
                1,
                Some(c.into()),
            ));
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, Some(c.into()), 2, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

// The part of darksynchop both Dark Tuners share, up to CompleteProcedure; returns the Synchro Monster summoned.
pub fn darksynchsummon(duel: &mut Duel, a: &Args) -> Script<Option<CardId>> {
    let (e, tp) = (a.e, a.tp);
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_SYNCHRO);
    let c = e.get_handler(duel);
    if !pg.is_empty() || !c.is_relate_to_effect(duel, e) || !c.is_faceup(duel) {
        return Ok(None);
    }
    let lv = c.get_level(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let filter = |sc, duel: &Duel| nontunerfilter(sc, duel, e, tp, lv, c);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?
        .get_first()
    else {
        return Ok(None);
    };
    duel.hint_selection(sc);
    let lv = lv - sc.get_level(duel);
    if duel.release(CardSet::from_iter([c, sc]), REASON_EFFECT)? != 2 {
        return Ok(None);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |sc, duel: &Duel| darksyncfilter(sc, duel, e, tp, lv, None);
    let sync = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first();
    let Some(sync) = sync else { return Ok(None) };
    if duel.special_summon(sync, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(None);
    }
    sync.complete_procedure(duel);
    Ok(Some(sync))
}

fn darksynchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if darksynchsummon(duel, a)?.is_some()
        && duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
