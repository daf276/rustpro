// Zoroa, the Magistus Conflagrant Calamity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 95911373;

pub const LISTED_SERIES: &[u16] = &[SET_MAGISTUS];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_SPELLCASTER, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck and the Graveyard, Synchro Summoned with Zoroa, the Magistus of Flame (a Spellcaster Tuner) and Crowley,
// with Rilliona, the Wondrous Magistus Witch Wardrobe in the Extra Deck to equip, and Magistus on the field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (36099130, MZONE),
    (875572, MZONE),
    (875572, MZONE),
    (74689476, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_TRIGGER);
    e2.set_target_range(duel, 0, LOCATION_ALL);
    e2.set_value(duel, 1);
    e2.set_target_card(duel, attg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| c.is_set_card(duel, &[SET_MAGISTUS]), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.is_location(duel, LOCATION_MZONE)
        && duel.get_location_count(tp, LOCATION_SZONE) > 0)
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(
        tp,
        |c, duel| c.is_set_card(duel, &[SET_MAGISTUS]),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(ec) = g.get_first() {
        duel.equip(tp, ec, c)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(c.into()));
        ec.register_effect(duel, e1);
    }
    Ok(())
}

fn atfilter(c: CardId, duel: &Duel, ty: u32) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MAGISTUS]) && c.is_original_type(duel, TYPE_MONSTER) && c.is_original_type(duel, ty)
}

fn attg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let ty = c.get_type(duel) & (TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK);
    ty != 0
        && duel.is_existing_matching_card(
            |c, duel| atfilter(c, duel, ty),
            e.get_handler_player(duel),
            LOCATION_STZONE,
            0,
            1,
            None,
        )
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MAGISTUS]) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && filter(chkc, duel, tp));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(ft > -1
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(|c, duel| filter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
