// Buzzsaw Shark
use crate::prelude::*;
use crate::testing::{DECK, MZONE};

const ID: u32 = 7150545;

// On the field with face-up Level 4 WATER monsters (Great White) and other Level 4 Fish (Tongyo, Crazy Fish) in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13429800, MZONE),
    (13429800, MZONE),
    (69572024, DECK),
    (69572024, DECK),
    (53713014, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_XYZ_LEVEL);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::XyzLevels(xyzlv));
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_attribute(duel, ATTRIBUTE_WATER)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_DECK, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    c.is_race(duel, RACE_FISH)
        && c.get_level(duel) == tc.get_level(duel)
        && !c.is_code(duel, tc.get_code(duel))
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, tc), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
        {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3302);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            sc.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
    }
    let ge1 = duel.create_effect(c);
    ge1.set_type(duel, EFFECT_TYPE_FIELD);
    ge1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    ge1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    ge1.set_description(duel, aux::stringid(ID, 1));
    ge1.set_target_range(duel, 1, 0);
    ge1.set_target_spsummon(duel, splimit);
    ge1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, lizfilter);
    Ok(())
}

fn xyzlv(duel: &Duel, e: EffectId, _: CardId, rc: CardId) -> Vec<i32> {
    let level = e.get_handler(duel).get_level(duel) as i32;
    if rc.is_attribute(duel, ATTRIBUTE_WATER) {
        vec![5, 3, level]
    } else {
        vec![level]
    }
}

pub fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    !c.is_type(duel, TYPE_XYZ) && c.is_location(duel, LOCATION_EXTRA)
}

pub fn lizfilter(duel: &Duel, _: EffectId, c: CardId) -> bool {
    !c.is_original_type(duel, TYPE_XYZ)
}
