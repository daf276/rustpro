// Number 29: Mannequin Cat
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 54191698;
pub const XYZ_NUMBER: u32 = 29;

// In the Extra Deck, Xyz Summoned with two Level 2 monsters, with the opponent's monsters in the Graveyard to Special
// Summon for them and monsters to Special Summon for itself.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (7892180, MZONE),
    (59053232, MZONE),
    (26976414, MZONE | OPPONENT),
    (1761063, GRAVE | OPPONENT),
    (2468169, GRAVE | OPPONENT),
    (32864, 0),
    (6297941, 0),
    (7805359, 0),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - a.tp), 1, None))
    });
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && spfilter1(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
                && duel.is_existing_target(|c, duel| spfilter1(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter1(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, 1 - a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter3(sc, duel, e, tp, c),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter3(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    (c.is_attribute(duel, tc.get_attribute(duel)) || c.is_race(duel, tc.get_race(duel)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && spfilter2(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter2(c, duel, e, tp), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| spfilter2(c, duel, e, tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            |c, duel| spfilter3(c, duel, e, tp, tc),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
