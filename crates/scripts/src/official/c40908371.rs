// Azure-Eyes Silver Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 40908371;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_NORMAL, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned (Level 9) with Galaxy Serpent, Baby Dragon and Luster Dragon, with a Normal Monster
// in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (11066358, MZONE),
    (11066358, MZONE),
    (88819587, MZONE),
    (88819587, MZONE),
    (11091375, MZONE),
    (11091375, MZONE),
    (88819587, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_DRAGON)),
        tp,
        LOCATION_MZONE,
        0,
        None,
    );
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3009);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_value(duel, 1);
        e2.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
