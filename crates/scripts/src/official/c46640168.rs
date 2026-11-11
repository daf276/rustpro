// Fiendsmith's Lacrima
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 46640168;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter); 2];
}

// In the Extra Deck, Fusion Summoned with Fiendsmith's Tract from two Evil HERO Adusted Gold, which are also in the GY to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (98567237, GRAVE),
    (13650422, HAND),
    (13650422, MZONE),
    (13650422, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, -600);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e2.set_target(duel, thsptg);
    e2.set_operation(duel, thspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, super::c60764609::spcost);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, stock::damage);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp) && c.is_race_full(duel, RACE_FIEND, Some(fc), sumtype, tp)
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_FIEND)
        && c.is_faceup(duel)
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
            && chkc.is_controler(duel, tp)
            && thspfilter(chkc, duel, e, tp, ft));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c, duel| thspfilter(c, duel, e, tp, ft),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_target(
        tp,
        |c, duel| thspfilter(c, duel, e, tp, ft),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    }
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        tc,
        tp,
        |tc, duel| duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 3),
    )
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1200);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1200);
    Ok(false)
}
